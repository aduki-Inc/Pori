pub mod api;
pub mod dashboard;
pub mod static_files;

use anyhow::{Context, Result};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::{AppState, DashboardEvent};
use dashboard::DashboardService;

/// Run the dashboard HTTP server
pub async fn run_dashboard_server(
    app_state: Arc<AppState>,
    mut event_rx: mpsc::UnboundedReceiver<DashboardEvent>,
) -> Result<()> {
    let bind_addr = app_state.settings.dashboard.bind_address.clone();
    let port = app_state.settings.dashboard.port;
    let addr: SocketAddr = format!("{}:{}", bind_addr, port)
        .parse()
        .context("Invalid dashboard address")?;

    info!("Starting dashboard server on http://{}", addr);

    // Create dashboard service
    let service = Arc::new(DashboardService::new(app_state.clone()));

    // Start event handler task
    let event_task = {
        let service = service.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                service.handle_event(event).await;
            }
            debug!("Dashboard event handler stopped");
        })
    };

    // Create service factory
    let make_svc = make_service_fn(move |_conn| {
        let service = service.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let service = service.clone();
                async move { service.handle_request(req).await }
            }))
        }
    });

    // Create server
    let server = Server::bind(&addr).serve(make_svc);

    info!("Dashboard server listening on http://{}", addr);

    // Run server and event handler
    tokio::select! {
        result = server => {
            if let Err(e) = result {
                error!("Dashboard server error: {}", e);
            }
        }
        result = event_task => {
            if let Err(e) = result {
                error!("Dashboard event task panicked: {}", e);
            }
        }
    }

    info!("Dashboard server stopped");
    Ok(())
}
