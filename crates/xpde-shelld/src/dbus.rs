//! Session D-Bus: `org.xpde.Shell1` and `org.xpde.Web1` on connection name `org.xpde.Shell`.

use std::sync::Arc;

use zbus::{connection, interface};
use zbus::fdo::{Error as FdoError, Result as FdoResult};

use crate::launcher;
use crate::web::state::WebState;

pub struct ShellDBus;

#[interface(name = "org.xpde.Shell1")]
impl ShellDBus {
    fn ping(&self) -> String {
        "pong".to_string()
    }

    fn list_apps(&self) -> FdoResult<String> {
        let apps = xpde_desktop::list_apps();
        serde_json::to_string(&apps).map_err(|e| FdoError::Failed(e.to_string()))
    }

    fn launch_desktop(&self, id: String, _action: String) -> FdoResult<()> {
        launcher::launch_desktop(&id).map_err(|e| FdoError::Failed(e.to_string()))
    }
}

pub struct WebDBus {
    state: Arc<WebState>,
}

#[interface(name = "org.xpde.Web1")]
impl WebDBus {
    fn ping(&self) -> String {
        "pong".to_string()
    }

    async fn probe_url(&self, url: String) -> FdoResult<String> {
        let r = self.state.probe_url(&url).await;
        serde_json::to_string(&r).map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn list_pinned(&self) -> FdoResult<String> {
        let v = self.state.list_pinned_summaries().await;
        serde_json::to_string(&v).map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn pin_site(&self, spec_json: String) -> FdoResult<String> {
        self.state
            .pin_site_json(&spec_json)
            .await
            .map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn unpin_site(&self, site_id: String) -> FdoResult<()> {
        self.state
            .unpin_site(&site_id)
            .await
            .map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn get_sitemap(&self, site_id: String, max_depth: u32) -> FdoResult<String> {
        let tree = self
            .state
            .get_sitemap_tree(&site_id, max_depth)
            .await
            .map_err(|e| FdoError::Failed(e.to_string()))?;
        serde_json::to_string(&tree).map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn refresh_site(&self, site_id: String) -> FdoResult<()> {
        self.state
            .refresh_site(&site_id)
            .await
            .map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn search(&self, query: String) -> FdoResult<String> {
        let apps = xpde_desktop::list_apps();
        let hits = self.state.search(&query, &apps).await;
        serde_json::to_string(&hits).map_err(|e| FdoError::Failed(e.to_string()))
    }

    fn resolve_address(&self, input: String) -> FdoResult<String> {
        let r = self.state.resolve_address(&input);
        serde_json::to_string(&r).map_err(|e| FdoError::Failed(e.to_string()))
    }

    fn open_site(&self, site_id: String, path: String) -> FdoResult<()> {
        self.state
            .open_site(&site_id, &path)
            .map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn list_feeds(&self) -> FdoResult<String> {
        let v = self.state.list_feed_summaries().await;
        serde_json::to_string(&v).map_err(|e| FdoError::Failed(e.to_string()))
    }

    async fn feed_items(&self, feed_id: String, limit: u32) -> FdoResult<String> {
        let items = self
            .state
            .feed_items(&feed_id, limit as usize)
            .await
            .map_err(|e| FdoError::Failed(e.to_string()))?;
        serde_json::to_string(&items).map_err(|e| FdoError::Failed(e.to_string()))
    }
}

pub async fn serve(state: Arc<WebState>) -> zbus::Result<()> {
    let shell = ShellDBus;
    let web = WebDBus { state };
    let _conn = connection::Builder::session()?
        .name("org.xpde.Shell")?
        .serve_at("/org/xpde/Shell1", shell)?
        .serve_at("/org/xpde/Web1", web)?
        .build()
        .await?;
    std::future::pending::<()>().await;
    Ok(())
}
