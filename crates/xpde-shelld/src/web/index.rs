use std::path::Path;

use xpde_ipc::{AppEntry, FederatedHit, SiteSummary};

use super::search::{all_provider_dirs, apply_template, load_providers};

pub fn federated_hits(
    sites: &[SiteSummary],
    apps: &[AppEntry],
    query: &str,
    q_lower: &str,
    user_provider_dir: &Path,
) -> Vec<FederatedHit> {
    let mut hits = Vec::new();

    for app in apps {
        if app.name.to_ascii_lowercase().contains(q_lower)
            || app.id.to_ascii_lowercase().contains(q_lower)
        {
            hits.push(FederatedHit {
                kind: "app".into(),
                title: app.name.clone(),
                subtitle: Some(app.id.clone()),
                action: format!("launch:desktop:{}", app.id),
            });
        }
    }

    for site in sites {
        if site.title.to_ascii_lowercase().contains(q_lower)
            || site.url.to_ascii_lowercase().contains(q_lower)
        {
            hits.push(FederatedHit {
                kind: "site".into(),
                title: site.title.clone(),
                subtitle: Some(site.url.clone()),
                action: format!("open-site:{}:", site.id),
            });
        }
    }

    let dirs = all_provider_dirs(user_provider_dir);
    let providers = load_providers(&dirs);
    let parts: Vec<&str> = query.split_whitespace().collect();
    if parts.len() >= 2 {
        let kw = parts[0].to_ascii_lowercase();
        let rest = parts[1..].join(" ");
        for p in &providers {
            if p.keyword.eq_ignore_ascii_case(&kw) {
                if let Ok(url) = apply_template(&p.template, &rest) {
                    hits.push(FederatedHit {
                        kind: "search".into(),
                        title: format!("{} — {}", p.name, rest),
                        subtitle: Some(url.clone()),
                        action: format!("open-url:{url}"),
                    });
                }
                break;
            }
        }
    }

    hits
}
