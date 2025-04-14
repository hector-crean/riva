use riva_ws_server::{WsServer, error::WsServerError};
use tracing_subscriber::{FmtSubscriber, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), WsServerError> {
    // Configure the subscriber with a filter that enables different log levels
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("riva_ws_server=debug".parse().unwrap()))
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Now these log messages will be visible based on the configured level
    tracing::debug!("Starting WebSocket server in debug mode");
    tracing::info!("WebSocket server initializing");

    let server = WsServer::new();
    tracing::info!("Server starting on port 5555");
    server.run(5555).await?;

    Ok(())
}