use std::path::PathBuf;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct XpdePaths {
    pub config: PathBuf,
    pub web_objects: PathBuf,
    pub search_providers: PathBuf,
    pub cache_web: PathBuf,
    pub state_web: PathBuf,
    pub applications: PathBuf,
}

impl XpdePaths {
    pub fn detect() -> Result<Self> {
        let cfg = dirs::config_dir()
            .context("no config directory (XDG)")?
            .join("xpde");
        let web_objects = cfg.join("web-objects");
        let search_providers = cfg.join("search-providers");
        let cache_web = dirs::cache_dir()
            .context("no cache directory (XDG)")?
            .join("xpde")
            .join("web");
        let state_web = dirs::data_local_dir()
            .context("no local data directory (XDG)")?
            .join("xpde")
            .join("web");
        let applications = dirs::data_dir()
            .context("no data directory (XDG)")?
            .join("applications");
        Ok(Self {
            config: cfg,
            web_objects,
            search_providers,
            cache_web,
            state_web,
            applications,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.web_objects)?;
        std::fs::create_dir_all(&self.search_providers)?;
        std::fs::create_dir_all(&self.cache_web)?;
        std::fs::create_dir_all(&self.state_web)?;
        std::fs::create_dir_all(&self.applications)?;
        Ok(())
    }
}
