use anyhow::Result;

use crate::web::state::WebState;

pub async fn serve() -> Result<()> {
    let state = WebState::new().await?;
    crate::dbus::serve(state)
        .await
        .map_err(|e| anyhow::anyhow!("D-Bus server: {e}"))?;
    Ok(())
}
