pub mod client;
pub mod forwarder;
pub mod messages;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::{proxy_log, AppState};
use messages::HttpMessage;

/// Run the HTTP proxy forwarder component
pub async fn run_proxy_forwarder(
    app_state: Arc<AppState>,
    message_rx: mpsc::UnboundedReceiver<HttpMessage>,
) -> Result<()> {
    proxy_log!("Starting HTTP proxy forwarder");

    // Create proxy forwarder
    let forwarder = forwarder::ProxyForwarder::new(app_state.clone())?;

    // Start forwarder
    forwarder.run(message_rx).await?;

    proxy_log!("Proxy forwarder stopped");
    Ok(())
}
