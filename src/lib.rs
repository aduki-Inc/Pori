pub mod config;
pub mod server;
pub mod proxy;
pub mod websocket;
pub mod logging;
pub mod utils;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error};

use config::settings::AppSettings;

/// Shared application state
pub struct AppState {
    pub settings: AppSettings,
    pub dashboard_tx: mpsc::UnboundedSender<DashboardEvent>,
    pub proxy_tx: mpsc::UnboundedSender<proxy::messages::ProxyMessage>,
    pub websocket_tx: mpsc::UnboundedSender<websocket::messages::TunnelMessage>,
    pub stats: Arc<RwLock<AppStats>>,
}

/// Dashboard events for real-time updates
#[derive(Debug, Clone)]
pub enum DashboardEvent {
    RequestForwarded(String),
    ResponseReceived(u16, usize),
    Error(String),
    ConnectionStatus(ConnectionStatus),
    Statistics(AppStats),
}

/// Connection status enumeration
#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
    Error(String),
}

/// Application statistics
#[derive(Debug, Clone, Default)]
pub struct AppStats {
    pub requests_processed: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub bytes_forwarded: u64,
    pub uptime_seconds: u64,
    pub websocket_reconnects: u64,
    pub connection_status: String,
}

impl AppState {
    pub fn new(settings: AppSettings) -> (Self, AppChannels) {
        let (dashboard_tx, dashboard_rx) = mpsc::unbounded_channel();
        let (proxy_tx, proxy_rx) = mpsc::unbounded_channel();
        let (websocket_tx, websocket_rx) = mpsc::unbounded_channel();

        let state = Self {
            settings,
            dashboard_tx,
            proxy_tx,
            websocket_tx,
            stats: Arc::new(RwLock::new(AppStats::default())),
        };

        let channels = AppChannels {
            dashboard_rx,
            proxy_rx,
            websocket_rx,
        };

        (state, channels)
    }

    pub async fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut AppStats),
    {
        let mut stats = self.stats.write().await;
        updater(&mut *stats);
    }

    pub async fn get_stats(&self) -> AppStats {
        self.stats.read().await.clone()
    }
}

/// Channel receivers for component communication
pub struct AppChannels {
    pub dashboard_rx: mpsc::UnboundedReceiver<DashboardEvent>,
    pub proxy_rx: mpsc::UnboundedReceiver<proxy::messages::ProxyMessage>,
    pub websocket_rx: mpsc::UnboundedReceiver<websocket::messages::TunnelMessage>,
}

/// Initialize and run the application
pub async fn run_application(settings: AppSettings) -> Result<()> {
    info!("Starting tunnel client application");

    // Create application state and channels
    let (app_state, channels) = AppState::new(settings);
    let app_state = Arc::new(app_state);

    // Start application components concurrently
    let dashboard_task = if !app_state.settings.no_dashboard {
        Some(tokio::spawn({
            let state = app_state.clone();
            async move {
                if let Err(e) = server::run_dashboard_server(state, channels.dashboard_rx).await {
                    error!("Dashboard server error: {}", e);
                }
            }
        }))
    } else {
        None
    };

    let proxy_task = tokio::spawn({
        let state = app_state.clone();
        async move {
            if let Err(e) = proxy::run_proxy_forwarder(state, channels.proxy_rx).await {
                error!("Proxy forwarder error: {}", e);
            }
        }
    });

    let websocket_task = tokio::spawn({
        let state = app_state.clone();
        async move {
            if let Err(e) = websocket::run_websocket_client(state, channels.websocket_rx).await {
                error!("WebSocket client error: {}", e);
            }
        }
    });

    // Wait for shutdown signal
    let shutdown_task = tokio::spawn(async {
        utils::signals::wait_for_shutdown().await;
        info!("Shutdown signal received");
    });

    // Wait for any task to complete (which should only happen on shutdown or error)
    tokio::select! {
        _ = shutdown_task => info!("Application shutting down gracefully"),
        result = proxy_task => {
            if let Err(e) = result {
                error!("Proxy task panicked: {}", e);
            }
        }
        result = websocket_task => {
            if let Err(e) = result {
                error!("WebSocket task panicked: {}", e);
            }
        }
        result = async {
            if let Some(task) = dashboard_task {
                task.await
            } else {
                // If no dashboard, wait forever
                std::future::pending().await
            }
        } => {
            if let Err(e) = result {
                error!("Dashboard task panicked: {}", e);
            }
        }
    }

    info!("Application stopped");
    Ok(())
}
