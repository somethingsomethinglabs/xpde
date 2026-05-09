//! Opens a pinned site's URL (optional `--url` override) in the default browser / OS handler.
//! A full embedded WebKit shell can replace `open::that` later.

use anyhow::{Context, Result};
use clap::Parser;
use xpde_ipc::PinnedSiteConfig;

#[derive(Parser)]
#[command(name = "xpde-webview")]
struct Cli {
    #[arg(long)]
    site_id: String,
    #[arg(long)]
    url: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg_dir = dirs::config_dir().context("no XDG config directory")?;
    let path = cfg_dir
        .join("xpde/web-objects")
        .join(format!("{}.toml", cli.site_id));
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {:?}", path))?;
    let cfg: PinnedSiteConfig = toml::from_str(&raw).context("parse site TOML")?;
    let open_url = cli.url.unwrap_or(cfg.url);
    open::that(&open_url).with_context(|| format!("open {}", open_url))?;
    Ok(())
}
