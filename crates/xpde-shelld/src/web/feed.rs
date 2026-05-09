use anyhow::Result;
use xpde_ipc::FeedItem;

use super::cache::HttpCache;

pub async fn fetch_feed_items(cache: &HttpCache, feed_url: &str, limit: usize) -> Result<Vec<FeedItem>> {
    let fetched = cache.fetch(feed_url).await?;
    if !(200..300).contains(&fetched.status) {
        anyhow::bail!("feed HTTP {}", fetched.status);
    }
    let parsed = feed_rs::parser::parse(&fetched.bytes[..])
        .map_err(|e| anyhow::anyhow!("feed parse: {e}"))?;
    let mut out = Vec::new();
    for e in parsed.entries.iter().take(limit) {
        let title = e
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_else(|| "(untitled)".into());
        let link = e
            .links
            .first()
            .map(|l| l.href.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| feed_url.to_string());
        let published = e.published.or(e.updated).map(|d| d.to_rfc3339());
        out.push(FeedItem {
            title,
            link,
            published,
        });
    }
    Ok(out)
}
