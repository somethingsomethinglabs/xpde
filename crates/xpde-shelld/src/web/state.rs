use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use url::Url;
use xpde_ipc::{
    FeedSubscriptionToml, PinSiteSpec, PinnedSiteConfig, ProbeResult, SearchBindingToml,
    SiteSummary, SitemapConfigToml,
};

use super::cache::HttpCache;
use super::paths::XpdePaths;
use super::pin::{self, site_id_for_url};
use super::search::all_provider_dirs;
use super::sitemap;

#[derive(Debug)]
pub struct WebState {
    pub paths: XpdePaths,
    http: HttpCache,
    sites: RwLock<HashMap<String, PinnedSiteConfig>>,
}

impl WebState {
    pub async fn new() -> Result<Arc<Self>> {
        let paths = XpdePaths::detect()?;
        paths.ensure_dirs()?;
        let http = HttpCache::new(paths.cache_web.clone())?;
        let sites = RwLock::new(load_all_sites(&paths.web_objects)?);
        Ok(Arc::new(Self { paths, http, sites }))
    }

    pub async fn probe_url(&self, url: &str) -> ProbeResult {
        pin::probe_url(&self.http, url).await
    }

    pub async fn list_pinned_summaries(&self) -> Vec<SiteSummary> {
        let g = self.sites.read().await;
        let mut v: Vec<SiteSummary> = g
            .values()
            .map(|c| SiteSummary {
                id: c.id.clone(),
                url: c.url.clone(),
                title: c.title.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        v
    }

    pub async fn pin_site_json(&self, spec_json: &str) -> Result<String> {
        let spec: PinSiteSpec = serde_json::from_str(spec_json).context("PinSiteSpec JSON")?;
        let id = site_id_for_url(&spec.url);
        let probe = self.probe_url(&spec.url).await;
        if let Some(e) = &probe.error {
            if probe.status_code.is_none() {
                anyhow::bail!("probe failed: {e}");
            }
        }

        let title = spec
            .title
            .clone()
            .or_else(|| probe.title.clone())
            .or_else(|| probe.manifest.as_ref().and_then(|m| m.short_name.clone()))
            .or_else(|| probe.manifest.as_ref().and_then(|m| m.name.clone()))
            .unwrap_or_else(|| id.clone());

        let mut sitemap_urls = probe.sitemap_urls.clone();
        sitemap_urls.extend(probe.robots_sitemap_hints.clone());
        sitemap_urls.sort();
        sitemap_urls.dedup();
        let primary_sitemap = sitemap_urls.first().cloned();

        let mut feeds = Vec::new();
        if spec.include_feeds {
            let chosen: Vec<_> = if let Some(ref subset) = spec.feed_urls {
                probe
                    .feed_urls
                    .iter()
                    .filter(|f| subset.iter().any(|u| u == &f.href))
                    .collect()
            } else {
                probe.feed_urls.iter().collect()
            };
            for (i, f) in chosen.iter().enumerate() {
                feeds.push(FeedSubscriptionToml {
                    id: format!("{id}-feed-{i}"),
                    url: f.href.clone(),
                    title: f.title.clone().unwrap_or_else(|| "Feed".into()),
                    poll_minutes: 30,
                    include_in_start_menu: true,
                });
            }
        }

        let sitemap_cfg = if spec.include_sitemap {
            Some(SitemapConfigToml {
                present: primary_sitemap.is_some(),
                source: primary_sitemap,
                max_depth: spec.sitemap_max_depth,
                include_in_start_menu: true,
            })
        } else {
            None
        };

        let search_cfg = spec.register_search_keyword.map(|kw| SearchBindingToml {
            opensearch: probe.opensearch_urls.first().cloned(),
            keyword: Some(kw),
        });

        let cfg = PinnedSiteConfig {
            id: id.clone(),
            url: spec.url.clone(),
            title: title.clone(),
            manifest: probe.manifest.clone(),
            sitemap: sitemap_cfg,
            feeds,
            search: search_cfg,
        };

        let toml_str = toml::to_string_pretty(&cfg)?;
        let path = self.paths.web_objects.join(format!("{id}.toml"));
        fs::write(&path, &toml_str)?;

        let desktop_path = self
            .paths
            .applications
            .join(format!("xpde-web-{id}.desktop"));
        pin::write_desktop_file(&desktop_path, &title, &id)?;

        self.sites.write().await.insert(id.clone(), cfg);

        Ok(id)
    }

    pub async fn unpin_site(&self, site_id: &str) -> Result<()> {
        let path = self.paths.web_objects.join(format!("{site_id}.toml"));
        let _ = fs::remove_file(&path);
        let desktop_path = self
            .paths
            .applications
            .join(format!("xpde-web-{site_id}.desktop"));
        let _ = fs::remove_file(&desktop_path);
        self.sites.write().await.remove(site_id);
        let site_dir = self.paths.state_web.join(site_id);
        let _ = fs::remove_dir_all(&site_dir);
        Ok(())
    }

    pub async fn get_sitemap_tree(&self, site_id: &str, max_depth: u32) -> Result<xpde_ipc::SitemapNode> {
        let cfg = self
            .sites
            .read()
            .await
            .get(site_id)
            .cloned()
            .with_context(|| format!("unknown site {site_id}"))?;
        let Some(sm) = cfg.sitemap.as_ref().and_then(|s| s.source.clone()) else {
            anyhow::bail!("site has no sitemap");
        };
        let base = Url::parse(&cfg.url)?;
        if !crate::web::pin::same_origin(&base, &sm) {
            anyhow::bail!("sitemap origin mismatch");
        }
        let cfg_limit = cfg.sitemap.as_ref().map(|s| s.max_depth).unwrap_or(2);
        let limit = max_depth.min(cfg_limit).max(1);

        let mut root = xpde_ipc::SitemapNode {
            loc: cfg.url.clone(),
            title: Some(cfg.title.clone()),
            children: Vec::new(),
        };
        let mut visited = HashSet::new();
        root.children = fetch_sitemap_level(
            &self.http,
            &sm,
            &base,
            1,
            limit,
            &mut visited,
        )
        .await?;
        Ok(root)
    }

    pub async fn refresh_site(&self, site_id: &str) -> Result<()> {
        let mut cfg = self
            .sites
            .read()
            .await
            .get(site_id)
            .cloned()
            .with_context(|| format!("unknown site {site_id}"))?;
        let probe = self.probe_url(&cfg.url).await;
        cfg.manifest = probe.manifest;
        let path = self.paths.web_objects.join(format!("{site_id}.toml"));
        fs::write(&path, toml::to_string_pretty(&cfg)?)?;
        self.sites.write().await.insert(site_id.to_string(), cfg);
        Ok(())
    }

    pub async fn search(&self, query: &str, apps: &[xpde_ipc::AppEntry]) -> Vec<xpde_ipc::FederatedHit> {
        let q_lower = query.to_ascii_lowercase();
        let sites = self.list_pinned_summaries().await;
        super::index::federated_hits(
            &sites,
            apps,
            query,
            &q_lower,
            &self.paths.search_providers,
        )
    }

    pub async fn resolve_address(&self, input: &str) -> xpde_ipc::ResolvedAddress {
        let t = input.trim();
        if let Some(rest) = t.strip_prefix("http://").or_else(|| t.strip_prefix("https://")) {
            return xpde_ipc::ResolvedAddress {
                scheme: "https".into(),
                target: format!("https://{rest}"),
                query: None,
            };
        }
        if let Some(u) = t.strip_prefix("feed://") {
            return xpde_ipc::ResolvedAddress {
                scheme: "feed".into(),
                target: format!("https://{u}"),
                query: None,
            };
        }
        let parts: Vec<&str> = t.split_whitespace().collect();
        if parts.len() >= 2 {
            let kw = parts[0];
            let rest = parts[1..].join(" ");
            let dirs = all_provider_dirs(&self.paths.search_providers);
            let providers = super::search::load_providers(&dirs);
            for p in providers {
                if p.keyword.eq_ignore_ascii_case(kw) {
                    if let Ok(url) = super::search::apply_template(&p.template, &rest) {
                        return xpde_ipc::ResolvedAddress {
                            scheme: "search".into(),
                            target: url,
                            query: Some(rest),
                        };
                    }
                }
            }
        }
        xpde_ipc::ResolvedAddress {
            scheme: "unknown".into(),
            target: t.to_string(),
            query: None,
        }
    }

    pub fn open_site(&self, site_id: &str, path_suffix: &str) -> Result<()> {
        let cfg_path = self.paths.web_objects.join(format!("{site_id}.toml"));
        let body = fs::read_to_string(&cfg_path).with_context(|| format!("read {cfg_path:?}"))?;
        let cfg: PinnedSiteConfig = toml::from_str(&body)?;
        let mut url = Url::parse(&cfg.url)?;
        if !path_suffix.is_empty() && path_suffix != "/" {
            let joined = url.join(path_suffix.trim_start_matches('/'))?;
            url = joined;
        }
        std::process::Command::new("xpde-webview")
            .args(["--site-id", site_id, "--url", url.as_str()])
            .spawn()
            .context("spawn xpde-webview")?;
        Ok(())
    }

    pub async fn feed_items(&self, feed_id: &str, limit: usize) -> Result<Vec<xpde_ipc::FeedItem>> {
        for site in self.sites.read().await.values() {
            for f in &site.feeds {
                if f.id == feed_id {
                    return super::feed::fetch_feed_items(&self.http, &f.url, limit).await;
                }
            }
        }
        anyhow::bail!("unknown feed id");
    }

    pub async fn list_feed_summaries(&self) -> Vec<xpde_ipc::FeedSummary> {
        let g = self.sites.read().await;
        let mut out = Vec::new();
        for site in g.values() {
            for f in &site.feeds {
                out.push(xpde_ipc::FeedSummary {
                    id: f.id.clone(),
                    url: f.url.clone(),
                    title: format!("{} — {}", site.title, f.title),
                });
            }
        }
        out
    }
}

async fn fetch_sitemap_level(
    http: &HttpCache,
    sitemap_url: &str,
    origin_base: &Url,
    depth: u32,
    max_depth: u32,
    visited: &mut HashSet<String>,
) -> Result<Vec<xpde_ipc::SitemapNode>> {
    if depth > max_depth || !visited.insert(sitemap_url.to_string()) {
        return Ok(Vec::new());
    }
    let fetched = http.fetch(sitemap_url).await?;
    if !(200..300).contains(&fetched.status) {
        return Ok(Vec::new());
    }
    let locs = sitemap::parse_locs(&fetched.bytes)?;
    let mut out = Vec::new();
    for loc in locs {
        if !Url::parse(&loc)
            .map(|u| u.origin() == origin_base.origin())
            .unwrap_or(false)
        {
            continue;
        }
        let mut children = Vec::new();
        if depth < max_depth && loc.to_ascii_lowercase().ends_with(".xml") {
            children = Box::pin(fetch_sitemap_level(
                http,
                &loc,
                origin_base,
                depth + 1,
                max_depth,
                visited,
            ))
            .await?;
        }
        out.push(xpde_ipc::SitemapNode {
            loc: loc.clone(),
            title: Some(short_title(&loc)),
            children,
        });
    }
    Ok(out)
}

fn load_all_sites(dir: &std::path::Path) -> Result<HashMap<String, PinnedSiteConfig>> {
    let mut map = HashMap::new();
    if !dir.is_dir() {
        return Ok(map);
    }
    for e in fs::read_dir(dir)? {
        let e = e?;
        let p = e.path();
        if !p.extension().is_some_and(|x| x == "toml") {
            continue;
        }
        let body = fs::read_to_string(&p)?;
        let cfg: PinnedSiteConfig = toml::from_str(&body)?;
        map.insert(cfg.id.clone(), cfg);
    }
    Ok(map)
}

fn short_title(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|u| {
            u.path_segments()
                .and_then(|mut s| s.next_back().map(|x| x.to_string()))
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| url.to_string())
}
