use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use xpde_ipc::AppEntry;

/// Lists `.desktop` applications from XDG data dirs (`applications/`).
pub fn list_apps() -> Vec<AppEntry> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for root in data_dirs() {
        let apps = root.join("applications");
        if apps.is_dir() {
            scan_applications_dir(&apps, &mut out, &mut seen);
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn data_dirs() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        v.push(PathBuf::from(home).join(".local/share"));
    }
    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        for part in xdg.split(':') {
            if !part.is_empty() {
                v.push(PathBuf::from(part));
            }
        }
    }
    if v.is_empty() || v.iter().all(|p| !p.join("applications").is_dir()) {
        v.push(PathBuf::from("/usr/local/share"));
        v.push(PathBuf::from("/usr/share"));
    }
    v
}

fn scan_applications_dir(dir: &Path, out: &mut Vec<AppEntry>, seen: &mut HashSet<String>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            scan_applications_dir(&p, out, seen);
            continue;
        }
        if !p.extension().is_some_and(|e| e == "desktop") {
            continue;
        }
        if let Some(app) = parse_desktop_file(&p) {
            if seen.insert(app.id.clone()) {
                out.push(app);
            }
        }
    }
}

fn parse_desktop_file(path: &Path) -> Option<AppEntry> {
    let data = fs::read_to_string(path).ok()?;
    let mut type_application = false;
    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut only_show_in: Option<&str> = None;
    let mut not_show_in: Option<&str> = None;
    let mut in_desktop_entry = false;

    for line in data.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_desktop_entry = line.eq_ignore_ascii_case("[desktop entry]");
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "Type" => type_application = val.trim().eq_ignore_ascii_case("application"),
            "Name" => name = Some(val.trim().to_string()),
            "Exec" => exec = Some(val.trim().to_string()),
            "NoDisplay" => no_display = val.trim().eq_ignore_ascii_case("true"),
            "Hidden" => hidden = val.trim().eq_ignore_ascii_case("true"),
            "OnlyShowIn" => only_show_in = Some(val.trim()),
            "NotShowIn" => not_show_in = Some(val.trim()),
            _ => {}
        }
    }

    if !type_application || no_display || hidden {
        return None;
    }

    let desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let desktops: Vec<&str> = desktop
        .split(':')
        .filter(|s| !s.is_empty())
        .collect();

    if let Some(o) = only_show_in {
        if !desktops.is_empty() {
            let allowed: Vec<&str> = o.split(';').filter(|s| !s.is_empty()).collect();
            if !allowed.iter().any(|d| desktops.contains(d)) {
                return None;
            }
        }
    }
    if let Some(n) = not_show_in {
        if !desktops.is_empty() {
            let blocked: Vec<&str> = n.split(';').filter(|s| !s.is_empty()).collect();
            if blocked.iter().any(|d| desktops.contains(d)) {
                return None;
            }
        }
    }

    let id = path.file_name()?.to_str()?.to_string();
    Some(AppEntry {
        id,
        name: name.unwrap_or_else(|| id.clone()),
        exec: exec.unwrap_or_default(),
    })
}
