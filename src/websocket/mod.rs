pub mod client;
pub mod messages;
pub mod reconnect;
pub mod tunnel;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, error};

use crate::AppState;
use messages::TunnelMessage;

/// Run the WebSocket client component
pub async fn run_websocket_client(
    app_state: Arc<AppState>,
    mut message_rx: mpsc::UnboundedReceiver<TunnelMessage>,
) -> Result<()> {
    info!("Starting WebSocket client");

    // Create WebSocket client
    let ws_client = client::WebSocketClient::new(app_state.clone())?;

    // Start client in background
    let client_handle = tokio::spawn({
        let client = ws_client.clone();
        async move {
            if let Err(e) = client.run().await {
                error!("WebSocket client error: {}", e);
            }
        }
    });

    // Handle outgoing messages
    let message_handle = tokio::spawn(async move {
        while let Some(message) = message_rx.recv().await {
            if let Err(e) = ws_client.send_message(message).await {
                error!("Failed to send WebSocket message: {}", e);
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        result = client_handle => {
            if let Err(e) = result {
                error!("WebSocket client task panicked: {}", e);
            }
        }
        result = message_handle => {
            if let Err(e) = result {
                error!("WebSocket message handler task panicked: {}", e);
            }
        }
    }

    info!("WebSocket client stopped");
    Ok(())
}
