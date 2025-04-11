use riva_ws_server::{WsServer, WsServerError};
use tracing_subscriber::FmtSubscriber;



#[tokio::main]
async fn main() -> Result<(), WsServerError> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;

    let server = WsServer::new();
    server.run(3000).await?;

    Ok(())
}