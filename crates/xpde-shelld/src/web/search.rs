use std::fs;
use std::path::Path;

use anyhow::Result;
use xpde_ipc::SearchProviderDef;

/// Load OpenSearch-style keyword providers from `*.toml` files in `dirs`.
pub fn load_providers(dirs: &[std::path::PathBuf]) -> Vec<SearchProviderDef> {
    let mut out = Vec::new();
    for d in dirs {
        let Ok(rd) = fs::read_dir(d) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if !p.extension().is_some_and(|x| x == "toml") {
                continue;
            }
            if let Ok(txt) = fs::read_to_string(&p) {
                if let Ok(def) = toml::from_str::<SearchProviderDef>(&txt) {
                    out.push(def);
                }
            }
        }
    }
    out.sort_by(|a, b| a.keyword.cmp(&b.keyword));
    out
}

pub fn apply_template(template: &str, query: &str) -> Result<String> {
    if !template.contains("{query}") {
        anyhow::bail!("provider template missing {{query}} placeholder");
    }
    let enc = urlencoding::encode(query);
    Ok(template.replace("{query}", &enc))
}

/// `/usr/share/xpde/search-providers` when present (installed session files).
pub fn system_provider_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("/usr/share/xpde/search-providers")
}

pub fn repo_provider_dir() -> Option<std::path::PathBuf> {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates
    p.pop(); // repo root
    let session = p.join("session/search-providers");
    if session.is_dir() {
        Some(session)
    } else {
        None
    }
}

pub fn all_provider_dirs(user_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut v = Vec::new();
    if let Some(repo) = repo_provider_dir() {
        v.push(repo);
    }
    let sys = system_provider_dir();
    if sys.is_dir() {
        v.push(sys);
    }
    v.push(user_dir.to_path_buf());
    v
}
