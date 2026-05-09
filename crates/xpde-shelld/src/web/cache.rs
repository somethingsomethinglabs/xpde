use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use sha2::{Digest, Sha256};

/// Minimal HTTP fetch helper with disk spill for large bodies.
pub struct HttpCache {
    client: Client,
    cache_dir: PathBuf,
}

impl HttpCache {
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!(
                "xpde-shelld/",
                env!("CARGO_PKG_VERSION"),
                " (+https://xpde.local)"
            ))
            .build()?;
        Ok(Self { client, cache_dir })
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    fn spill_path(&self, url: &str) -> PathBuf {
        let mut h = Sha256::new();
        h.update(url.as_bytes());
        let name = hex::encode(h.finalize());
        self.cache_dir.join(format!("{name}.bin"))
    }

    /// GET `url`, return status, final URL after redirects, and body bytes.
    pub async fn fetch(&self, url: &str) -> Result<Fetched> {
        let resp = self.client.get(url).send().await?;
        let status = resp.status().as_u16();
        let final_url = resp.url().to_string();
        let bytes = resp.bytes().await?.to_vec();
        let _ = std::fs::write(self.spill_path(url), &bytes);
        Ok(Fetched {
            status,
            final_url,
            bytes,
        })
    }
}

#[derive(Debug)]
pub struct Fetched {
    pub status: u16,
    pub final_url: String,
    pub bytes: Vec<u8>,
}
