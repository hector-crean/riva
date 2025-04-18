use riva_ws_server::{Application, ApplicationConfig};
use tracing_subscriber::{FmtSubscriber, EnvFilter};
use color_eyre::eyre;
use riva_ws_server::AppState;

#[tokio::main]
async fn main() -> eyre::Result<()> {

    color_eyre::install()?;
    dotenv::dotenv().ok();
    
    // Configure the subscriber with a filter that enables different log levels
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("riva_ws_server=debug".parse().unwrap()))
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Now these log messages will be visible based on the configured level
    tracing::debug!("Starting WebSocket server in debug mode");
    tracing::info!("WebSocket server initializing");


    let aws_key = std::env::var("AWS_ACCESS_KEY_ID").expect("Failed to get AWS key.");
    let aws_key_secret =
        std::env::var("AWS_SECRET_ACCESS_KEY").expect("Failed to get AWS secret key.");
    let s3_region = std::env::var("AWS_REGION").unwrap_or("eu-west-2".to_string());
    let aws_bucket = std::env::var("S3_BUCKET_NAME").expect("Failed to get AWS Bucket key");

    let config = ApplicationConfig {
        aws_key,
        aws_key_secret,
        s3_region,
        aws_bucket,
        surreal_url: "127.0.0.1:8000".to_string(),
    };

    let server = Application::new(config).await;

    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "5555".to_string())
    .parse::<u16>()?;

    tracing::info!("Server starting on port {}", port);

    server.run(port).await?;

    Ok(())
}