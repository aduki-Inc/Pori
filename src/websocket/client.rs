use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, instrument, warn};

use super::{messages::TunnelMessage, reconnect::ReconnectManager, tunnel::TunnelHandler};
use crate::{AppState, ConnectionStatus, DashboardEvent};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// WebSocket client for tunnel communication
#[derive(Clone)]
pub struct WebSocketClient {
    app_state: Arc<AppState>,
    tunnel_handler: Arc<TunnelHandler>,
    reconnect_manager: Arc<Mutex<ReconnectManager>>,
    message_queue: Arc<Mutex<Vec<TunnelMessage>>>,
    outbound_tx: Arc<Mutex<Option<mpsc::UnboundedSender<TunnelMessage>>>>,
}

impl WebSocketClient {
    /// Create new WebSocket client
    pub fn new(app_state: Arc<AppState>) -> Result<Self> {
        let tunnel_handler = Arc::new(TunnelHandler::new(app_state.clone()));

        let reconnect_manager = Arc::new(Mutex::new(
            ReconnectManager::new()
                .with_max_attempts(app_state.settings.websocket.max_reconnects)
                .with_base_delay(Duration::from_secs(1))
                .with_max_delay(Duration::from_secs(300)),
        ));

        Ok(Self {
            app_state,
            tunnel_handler,
            reconnect_manager,
            message_queue: Arc::new(Mutex::new(Vec::new())),
            outbound_tx: Arc::new(Mutex::new(None)),
        })
    }

    /// Main client run loop with reconnection
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        info!(
            "Starting WebSocket client for URL: {}",
            self.app_state.settings.websocket.url
        );

        loop {
            // Check if we should attempt connection
            let should_reconnect = {
                let manager = self.reconnect_manager.lock().await;
                manager.should_reconnect()
            };

            if !should_reconnect {
                error!("Maximum reconnection attempts reached, giving up");
                break;
            }

            // Update connection status
            let _ = self
                .app_state
                .dashboard_tx
                .send(DashboardEvent::ConnectionStatus(
                    ConnectionStatus::Connecting,
                ));

            // Attempt connection
            match self.connect_and_run().await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");

                    // Reset reconnection counter on successful connection
                    let mut manager = self.reconnect_manager.lock().await;
                    manager.reset();
                }
                Err(e) => {
                    error!("WebSocket connection failed: {}", e);

                    // Update connection status
                    let _ = self
                        .app_state
                        .dashboard_tx
                        .send(DashboardEvent::ConnectionStatus(
                            ConnectionStatus::Disconnected,
                        ));

                    // Calculate reconnection delay
                    let delay = {
                        let mut manager = self.reconnect_manager.lock().await;
                        manager.next_delay()
                    };

                    if delay > Duration::from_secs(0) {
                        info!("Waiting {:?} before reconnection attempt", delay);

                        // Update status to reconnecting
                        let _ = self
                            .app_state
                            .dashboard_tx
                            .send(DashboardEvent::ConnectionStatus(
                                ConnectionStatus::Reconnecting,
                            ));

                        tokio::time::sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        // Update final status
        let _ = self
            .app_state
            .dashboard_tx
            .send(DashboardEvent::ConnectionStatus(
                ConnectionStatus::Disconnected,
            ));

        Ok(())
    }

    /// Single connection attempt and message handling
    #[instrument(skip(self))]
    async fn connect_and_run(&self) -> Result<()> {
        info!(
            "Attempting WebSocket connection to {}",
            self.app_state.settings.websocket.url
        );

        // Establish WebSocket connection
        let (ws_stream, response) = tokio::time::timeout(
            self.app_state.settings.websocket.timeout,
            connect_async(self.app_state.settings.websocket.url.as_str()),
        )
        .await
        .context("Connection timeout")?
        .context("Failed to connect to WebSocket server")?;

        info!(
            "WebSocket connected, response status: {}",
            response.status()
        );

        // Split stream for concurrent read/write
        let (mut ws_sink, mut ws_stream) = ws_stream.split();

        // Create channel for outbound messages
        let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<TunnelMessage>();

        // Store the sender for external use
        {
            let mut tx_guard = self.outbound_tx.lock().await;
            *tx_guard = Some(outbound_tx);
        }

        // Send authentication message
        let auth_message = self.tunnel_handler.create_auth_message();
        let auth_json = auth_message.to_json()?;
        ws_sink
            .send(Message::Text(auth_json.into()))
            .await
            .context("Failed to send authentication message")?;

        // Setup ping task
        let ping_task = {
            let outbound_tx = self.outbound_tx.clone();
            let ping_interval = self.app_state.settings.websocket.ping_interval;

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(ping_interval);
                interval.tick().await; // Skip first tick

                loop {
                    interval.tick().await;

                    let ping_message = TunnelHandler::create_ping_message();
                    if let Some(ref tx) = *outbound_tx.lock().await {
                        if tx.send(ping_message).is_err() {
                            break;
                        }
                    }
                }
            })
        };

        // Process queued messages
        self.send_queued_messages().await?;

        // Main message handling loop
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                ws_message = ws_stream.next() => {
                    match ws_message {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.handle_incoming_message(msg).await {
                                error!("Error handling incoming message: {}", e);
                            }
                        }
                        Some(Err(e)) => {
                            error!("WebSocket stream error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                    }
                }

                // Handle outbound messages
                outbound_message = outbound_rx.recv() => {
                    match outbound_message {
                        Some(message) => {
                            if let Err(e) = self.send_message_to_stream(&mut ws_sink, message).await {
                                error!("Error sending message: {}", e);
                                break;
                            }
                        }
                        None => {
                            debug!("Outbound message channel closed");
                            break;
                        }
                    }
                }
            }
        }

        // Cleanup
        ping_task.abort();

        // Clear outbound sender
        {
            let mut tx_guard = self.outbound_tx.lock().await;
            *tx_guard = None;
        }

        Ok(())
    }

    /// Handle incoming WebSocket message
    #[instrument(skip(self, message))]
    async fn handle_incoming_message(&self, message: Message) -> Result<()> {
        match message {
            Message::Text(text) => {
                debug!("Received text message: {}", text);

                let tunnel_message =
                    TunnelMessage::from_json(&text).context("Failed to parse tunnel message")?;

                if let Some(response) = self.tunnel_handler.handle_message(tunnel_message).await? {
                    self.send_message(response).await?;
                }
            }

            Message::Binary(data) => {
                debug!("Received binary message ({} bytes)", data.len());

                let tunnel_message = TunnelMessage::from_binary(&data)
                    .context("Failed to parse binary tunnel message")?;

                if let Some(response) = self.tunnel_handler.handle_message(tunnel_message).await? {
                    self.send_message(response).await?;
                }
            }

            Message::Ping(_data) => {
                debug!("Received WebSocket ping");
                // tokio-tungstenite automatically handles pong responses
            }

            Message::Pong(_) => {
                debug!("Received WebSocket pong");
            }

            Message::Close(frame) => {
                info!("Received close frame: {:?}", frame);
                return Err(anyhow::anyhow!("WebSocket closed by server"));
            }

            Message::Frame(_) => {
                warn!("Received unexpected frame message");
            }
        }

        Ok(())
    }

    /// Send message to WebSocket stream
    async fn send_message_to_stream(
        &self,
        ws_sink: &mut futures_util::stream::SplitSink<WsStream, Message>,
        message: TunnelMessage,
    ) -> Result<()> {
        let ws_message = if message.has_binary_data() {
            let binary_data = message.to_binary()?;
            Message::Binary(binary_data.into())
        } else {
            let json_data = message.to_json()?;
            Message::Text(json_data.into())
        };

        ws_sink
            .send(ws_message)
            .await
            .context("Failed to send message to WebSocket")?;

        debug!("Sent {} message", message.message_type());
        Ok(())
    }

    /// Send message via WebSocket (queue if not connected)
    pub async fn send_message(&self, message: TunnelMessage) -> Result<()> {
        let tx_guard = self.outbound_tx.lock().await;

        if let Some(ref tx) = *tx_guard {
            // Connection is active, send immediately
            tx.send(message)
                .map_err(|_| anyhow::anyhow!("Failed to send message: channel closed"))?;
        } else {
            // Connection is not active, queue the message
            debug!(
                "Queueing message for later delivery: {}",
                message.message_type()
            );
            let mut queue = self.message_queue.lock().await;
            queue.push(message);
        }

        Ok(())
    }

    /// Send queued messages when connection is established
    async fn send_queued_messages(&self) -> Result<()> {
        let mut queue = self.message_queue.lock().await;
        let queued_count = queue.len();

        if queued_count > 0 {
            info!("Sending {} queued messages", queued_count);

            let tx_guard = self.outbound_tx.lock().await;
            if let Some(ref tx) = *tx_guard {
                for message in queue.drain(..) {
                    if let Err(e) = tx.send(message) {
                        error!("Failed to send queued message: {}", e);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> WebSocketStats {
        let manager = self.reconnect_manager.lock().await;
        let queue = self.message_queue.lock().await;
        let is_connected = self.outbound_tx.lock().await.is_some();

        WebSocketStats {
            is_connected,
            current_attempt: manager.current_attempt(),
            max_attempts: manager.max_attempts(),
            queued_messages: queue.len(),
            url: self.app_state.settings.websocket.url.to_string(),
        }
    }
}

/// WebSocket connection statistics
#[derive(Debug, Clone)]
pub struct WebSocketStats {
    pub is_connected: bool,
    pub current_attempt: u32,
    pub max_attempts: u32,
    pub queued_messages: usize,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{cli::CliArgs, settings::AppSettings};

    fn create_test_app_state() -> Arc<AppState> {
        let args = CliArgs {
            url: "ws://localhost:7616".parse().unwrap(),
            token: "test-token".to_string(),
            protocol: "http".to_string(),
            port: 3000,
            dashboard_port: 7616,
            log_level: "info".to_string(),
            config: None,
            no_dashboard: false,
            timeout: 30,
            max_reconnects: 0,
            verify_ssl: false,
            max_connections: 10,
        };

        let settings = AppSettings::from_cli(args).unwrap();
        let (app_state, _) = AppState::new(settings);
        Arc::new(app_state)
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let app_state = create_test_app_state();
        let client = WebSocketClient::new(app_state).unwrap();

        let stats = client.get_stats().await;
        assert!(!stats.is_connected);
        assert_eq!(stats.queued_messages, 0);
    }

    #[tokio::test]
    async fn test_message_queueing() {
        let app_state = create_test_app_state();
        let client = WebSocketClient::new(app_state).unwrap();

        let message = TunnelMessage::ping();
        client.send_message(message).await.unwrap();

        let stats = client.get_stats().await;
        assert_eq!(stats.queued_messages, 1);
    }
}
