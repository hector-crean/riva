use riva_ws_server::{error::WsServerError, room::{presentation::{PresentationCommand, PresentationEvent}, room_id::RoomId}, WsServer};
use socketioxide::extract::SocketRef;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use riva_ws_server::{RoomConfig, room::presentation::PresentationRoom, room::video::Video};
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<(), WsServerError> {
    // Configure the subscriber with a filter that enables different log levels
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("riva_ws_server=debug".parse().unwrap()),
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Now these log messages will be visible based on the configured level
    tracing::debug!("Starting WebSocket server in debug mode");
    tracing::info!("WebSocket server initializing");

    let mut server = WsServer::new();

    // Register presentation rooms
    server.register_room_type(
        "presentation",
        |config: RoomConfig| -> Result<PresentationRoom, String> {
            // Extract room_id from metadata or use a default
            let room_id = if let Some(value) = config.metadata.get("room_id") {
                if let Some(id_str) = value.as_str() {
                    id_str
                        .to_string()
                        .try_into()
                        .map_err(|_| "Invalid room_id format".to_string())?
                } else {
                    return Err("room_id must be a string".to_string());
                }
            } else {
                // This is just a placeholder - in practice you'd want to get this from somewhere
                RoomId::new("default_org", "default_presentation")
            };

            Ok(PresentationRoom {
                id: room_id,
                slides: Vec::new(),
                current_slide: 0,
                connected_users: HashSet::new(),
            })
        },
    );

    // Register video rooms
    server.register_room_type("video", |config: RoomConfig| -> Result<Video, String> {
        let room_id = if let Some(value) = config.metadata.get("room_id") {
            if let Some(id_str) = value.as_str() {
                id_str
                    .to_string()
                    .try_into()
                    .map_err(|_| "Invalid room_id format".to_string())?
            } else {
                return Err("room_id must be a string".to_string());
            }
        } else {
            RoomId::new("default_org", "default_video")
        };

        let video_url = config
            .metadata
            .get("video_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(Video::new(room_id, "system".to_string(), video_url))
    });

    // Register global command handlers if needed
    server
        .register_command_handler(
            |cmd: PresentationCommand, socket: &SocketRef| {
                // Global handler for presentation commands
                // This could be used for logging, analytics, etc.
                None::<PresentationEvent>
            },
        )
        .await;

    tracing::info!("Server starting on port 5555");
    server.run(5555).await?;

    Ok(())
}
