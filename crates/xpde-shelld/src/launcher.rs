use anyhow::{Context, Result};

/// Launch a `.desktop` id via `systemd-run --user --scope -- gtk-launch`.
pub fn launch_desktop(id: &str) -> Result<()> {
    let status = std::process::Command::new("systemd-run")
        .args(["--user", "--scope", "--", "gtk-launch", id])
        .status()
        .context("spawn systemd-run gtk-launch")?;
    if !status.success() {
        anyhow::bail!("gtk-launch failed with status {:?}", status.code());
    }
    Ok(())
}
