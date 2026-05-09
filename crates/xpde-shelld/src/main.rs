mod apps;
mod config;
mod dbus;
mod launcher;
mod shell;
mod tray;
mod web;
mod windows;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("xpde-shelld starting");
    let app_count = apps::list_apps().len();
    info!("indexed apps: {app_count}");
    shell::serve().await?;
    Ok(())
}
