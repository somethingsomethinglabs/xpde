use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
}

/// Result of probing a URL for installable web surfaces (manifest, sitemap, feeds, OpenSearch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub url: String,
    pub status_code: Option<u16>,
    pub final_url: Option<String>,
    pub title: Option<String>,
    pub favicon_url: Option<String>,
    pub manifest_url: Option<String>,
    pub manifest: Option<WebManifestSummary>,
    pub sitemap_urls: Vec<String>,
    pub feed_urls: Vec<FeedLink>,
    pub opensearch_urls: Vec<String>,
    pub robots_sitemap_hints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebManifestSummary {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub display: Option<String>,
    pub start_url: Option<String>,
    pub theme_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedLink {
    pub href: String,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteSummary {
    pub id: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapNode {
    pub loc: String,
    pub title: Option<String>,
    #[serde(default)]
    pub children: Vec<SitemapNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSummary {
    pub id: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub published: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedHit {
    /// `app`, `site`, `sitemap`, `feed`, `search`
    pub kind: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Opaque action hint for the shell, e.g. `launch:desktop:id`, `open-site:id`, `search:provider`
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedAddress {
    pub scheme: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinSiteSpec {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default = "default_true")]
    pub include_sitemap: bool,
    #[serde(default = "default_depth")]
    pub sitemap_max_depth: u32,
    #[serde(default = "default_true")]
    pub include_feeds: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_search_keyword: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_depth() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProviderDef {
    pub id: String,
    pub name: String,
    pub keyword: String,
    pub template: String,
}

/// On-disk shape under `~/.config/xpde/web-objects/<id>.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedSiteConfig {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub manifest: Option<WebManifestSummary>,
    #[serde(default)]
    pub sitemap: Option<SitemapConfigToml>,
    #[serde(default)]
    pub feeds: Vec<FeedSubscriptionToml>,
    #[serde(default)]
    pub search: Option<SearchBindingToml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapConfigToml {
    #[serde(default)]
    pub present: bool,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default = "default_depth")]
    pub max_depth: u32,
    #[serde(default = "default_true")]
    pub include_in_start_menu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSubscriptionToml {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(default = "default_poll")]
    pub poll_minutes: u64,
    #[serde(default = "default_true")]
    pub include_in_start_menu: bool,
}

fn default_poll() -> u64 {
    30
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchBindingToml {
    #[serde(default)]
    pub opensearch: Option<String>,
    #[serde(default)]
    pub keyword: Option<String>,
}
