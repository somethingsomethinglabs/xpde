use anyhow::Result;
use scraper::{Html, Selector};
use url::Url;
use xpde_ipc::{FeedLink, ProbeResult, WebManifestSummary};

use super::cache::HttpCache;
use super::sitemap;

pub async fn probe_url(cache: &HttpCache, url: &str) -> ProbeResult {
    let mut out = ProbeResult {
        url: url.to_string(),
        status_code: None,
        final_url: None,
        title: None,
        favicon_url: None,
        manifest_url: None,
        manifest: None,
        sitemap_urls: Vec::new(),
        feed_urls: Vec::new(),
        opensearch_urls: Vec::new(),
        robots_sitemap_hints: Vec::new(),
        error: None,
    };

    let Ok(base) = Url::parse(url) else {
        out.error = Some("invalid URL".into());
        return out;
    };

    match cache.fetch(url).await {
        Ok(fetched) => {
            out.status_code = Some(fetched.status);
            out.final_url = Some(fetched.final_url.clone());
            if !(200..300).contains(&fetched.status) {
                out.error = Some(format!("HTTP {}", fetched.status));
                return out;
            }
            let html = String::from_utf8_lossy(&fetched.bytes);
            enrich_from_html(&mut out, &base, &html);

            // Discover extra sitemap URLs from robots.txt (same origin only).
            if let Ok(robots) = robots_hints(cache, origin_root(&base).as_str()).await {
                out.robots_sitemap_hints = robots;
            }

            merge_unique(&mut out.sitemap_urls, guess_sitemaps(&base, &out));

            // Optional: fetch first manifest for summary fields.
            if let Some(ref rel) = out.manifest_url {
                if same_origin(&base, rel) {
                    if let Ok(m) = fetch_manifest_summary(cache, rel).await {
                        out.manifest = Some(m);
                    }
                }
            }
        }
        Err(e) => {
            out.error = Some(format!("{e:#}"));
        }
    }

    out
}

fn origin_root(base: &Url) -> Url {
    let mut u = base.clone();
    u.set_path("");
    u.set_query(None);
    u.set_fragment(None);
    u
}

pub(crate) fn same_origin(base: &Url, other: &str) -> bool {
    Url::parse(other)
        .ok()
        .map(|u| u.origin() == base.origin())
        .unwrap_or(false)
}

fn same_origin_str(base: &str, other: &str) -> bool {
    let Ok(a) = Url::parse(base) else {
        return false;
    };
    same_origin(&a, other)
}

fn merge_unique(into: &mut Vec<String>, more: Vec<String>) {
    for m in more {
        if !into.iter().any(|x| x == &m) {
            into.push(m);
        }
    }
}

fn guess_sitemaps(base: &Url, probe: &ProbeResult) -> Vec<String> {
    let mut v = Vec::new();
    let root = origin_root(base);
    if let Ok(u) = root.join("sitemap.xml") {
        v.push(u.to_string());
    }
    if let Ok(u) = root.join("sitemap_index.xml") {
        v.push(u.to_string());
    }
    for hint in &probe.robots_sitemap_hints {
        if !v.contains(hint) {
            v.push(hint.clone());
        }
    }
    v
}

async fn robots_hints(cache: &HttpCache, origin_root_url: &str) -> Result<Vec<String>> {
    let Ok(root) = Url::parse(origin_root_url) else {
        return Ok(Vec::new());
    };
    let robots_url = root.join("robots.txt")?;
    let fetched = cache.fetch(robots_url.as_str()).await?;
    if !(200..300).contains(&fetched.status) {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&fetched.bytes);
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix_ignore_ascii_case("sitemap:") {
            let u = rest.trim();
            if !u.is_empty() && same_origin_str(origin_root_url, u) {
                out.push(u.to_string());
            }
        }
    }
    Ok(out)
}

trait StripPrefixAscii {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str>;
}

impl StripPrefixAscii for str {
    fn strip_prefix_ignore_ascii_case(&self, prefix: &str) -> Option<&str> {
        let b = self.as_bytes();
        let p = prefix.as_bytes();
        if b.len() >= p.len() && self[..p.len()].eq_ignore_ascii_case(prefix) {
            Some(&self[p.len()..])
        } else {
            None
        }
    }
}

fn enrich_from_html(out: &mut ProbeResult, base: &Url, html: &str) {
    let Ok(title_sel) = Selector::parse("title") else {
        return;
    };
    let Ok(link_sel) = Selector::parse("link") else {
        return;
    };

    let fragment = Html::parse_document(html);
    if let Some(t) = fragment.select(&title_sel).next() {
        let text = t.inner_html().trim().to_string();
        if !text.is_empty() {
            out.title = Some(text);
        }
    }

    for el in fragment.select(&link_sel) {
        let rel = el.value().attr("rel").unwrap_or("").to_ascii_lowercase();
        let href = el.value().attr("href");
        let Some(href) = href else { continue };
        let Ok(abs) = base.join(href) else {
            continue;
        };
        let abs_s = abs.to_string();

        if rel.split_ascii_whitespace().any(|r| r == "manifest") {
            out.manifest_url.get_or_insert(abs_s.clone());
        }
        if rel.split_ascii_whitespace().any(|r| r == "search") {
            let type_attr = el.value().attr("type").unwrap_or("");
            if type_attr.contains("opensearch") || href.ends_with("xml") {
                out.opensearch_urls.push(abs_s.clone());
            }
        }
        if rel.split_ascii_whitespace().any(|r| r == "alternate") {
            let type_attr = el
                .value()
                .attr("type")
                .unwrap_or("")
                .to_ascii_lowercase();
            if type_attr.contains("rss")
                || type_attr.contains("atom")
                || type_attr.ends_with("+xml")
            {
                let title = el.value().attr("title").map(|s| s.to_string());
                out.feed_urls.push(FeedLink {
                    href: abs_s,
                    type_: Some(type_attr),
                    title,
                });
            }
        }
        if rel.split_ascii_whitespace().any(|r| r == "icon" || r == "shortcut icon") {
            out.favicon_url.get_or_insert(abs_s);
        }
    }

    // Last resort favicon.
    if out.favicon_url.is_none() {
        if let Ok(u) = origin_root(base).join("favicon.ico") {
            out.favicon_url = Some(u.to_string());
        }
    }
}

async fn fetch_manifest_summary(cache: &HttpCache, manifest_url: &str) -> Result<WebManifestSummary> {
    let fetched = cache.fetch(manifest_url).await?;
    if !(200..300).contains(&fetched.status) {
        anyhow::bail!("manifest HTTP {}", fetched.status);
    }
    let v: serde_json::Value = serde_json::from_slice(&fetched.bytes)?;
    Ok(WebManifestSummary {
        name: v
            .get("name")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        short_name: v
            .get("short_name")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        display: v
            .get("display")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        start_url: v
            .get("start_url")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        theme_color: v
            .get("theme_color")
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
    })
}

/// Stable filesystem-safe id from URL host + path.
pub fn site_id_for_url(url: &str) -> String {
    let Ok(u) = Url::parse(url) else {
        return "invalid".into();
    };
    let host = u.host_str().unwrap_or("site");
    let path = u.path().trim_matches('/');
    let mut base = format!(
        "{}-{}",
        slug(host),
        if path.is_empty() {
            "root".into()
        } else {
            slug(path)
        }
    );
    if base.len() > 80 {
        base.truncate(80);
    }
    base
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

pub fn write_desktop_file(
    path: &std::path::Path,
    title: &str,
    site_id: &str,
) -> Result<()> {
    let contents = format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Name={title}\n\
        Exec=xpde-webview --site-id {site_id}\n\
        Categories=XPDE-Web;\n\
        OnlyShowIn=xpde;\n"
    );
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)?;
    Ok(())
}
