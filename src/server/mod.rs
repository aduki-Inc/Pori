pub mod api;
pub mod dashboard;
pub mod static_files;

use anyhow::{Context, Result};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tracing::{debug, error};

use crate::{local_log, AppState, DashboardEvent};
use dashboard::DashboardService;

/// Run the dashboard HTTP server
pub async fn run_dashboard_server(
    app_state: Arc<AppState>,
    mut event_rx: mpsc::UnboundedReceiver<DashboardEvent>,
) -> Result<()> {
    let bind_addr = app_state.settings.dashboard.bind_address.clone();
    let port = app_state.settings.dashboard.port;
    let addr: SocketAddr = format!("{bind_addr}:{port}")
        .parse()
        .context("Invalid dashboard address")?;

    local_log!("Starting a dashboard server on http://{}", addr);

    // Create a dashboard service
    let service = Arc::new(DashboardService::new(app_state.clone()));

    // Start an event handler task
    let event_task = {
        let service = service.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                service.handle_event(event).await;
            }
            debug!("Dashboard event handler stopped");
        })
    };

    // Create TCP listener
    let listener = TcpListener::bind(addr)
        .await
        .context("Failed to bind to the dashboard address")?;

    local_log!("Dashboard server listening on http://{}", addr);

    // Accept connections
    let server_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let service = service.clone();
                    tokio::spawn(async move {
                        let io = TokioIo::new(stream);
                        if let Err(err) = http1::Builder::new()
                            .serve_connection(
                                io,
                                service_fn(move |req| {
                                    let service = service.clone();
                                    async move { service.handle_request(req).await }
                                }),
                            )
                            .await
                        {
                            error!("Error serving connection: {:?}", err);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    });

    // Run server and event handler
    tokio::select! {
        result = server_task => {
            if let Err(e) = result {
                error!("Dashboard server task panicked: {}", e);
            }
        }
        result = event_task => {
            if let Err(e) = result {
                error!("Dashboard event task panicked: {}", e);
            }
        }
    }

    local_log!("Dashboard server stopped");
    Ok(())
}
