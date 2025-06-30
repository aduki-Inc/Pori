use tokio::signal;
use tracing::{info, warn};

/// Wait for shutdown signals (Ctrl+C or SIGTERM)
pub async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received SIGTERM signal");
        },
    }

    info!("Starting graceful shutdown...");
}

/// Setup signal handlers for graceful shutdown
pub fn setup_signal_handlers() {
    // This function can be used to setup additional signal handling
    // if needed in the future
    
    #[cfg(unix)]
    {
        // On Unix systems, we might want to handle additional signals
        // like SIGUSR1 for log rotation, etc.
        tokio::spawn(async {
            let mut sigusr1 = signal::unix::signal(signal::unix::SignalKind::user_defined1())
                .expect("Failed to install SIGUSR1 handler");
            
            while sigusr1.recv().await.is_some() {
                info!("Received SIGUSR1 signal - could be used for log rotation");
            }
        });
    }
}

/// Force shutdown after timeout
pub async fn force_shutdown_after(timeout: std::time::Duration) {
    tokio::time::sleep(timeout).await;
    warn!("Force shutdown timeout reached, terminating process");
    std::process::exit(1);
}
