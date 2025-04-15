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
            Ok(PresentationRoom {
                id: config.room_id,
                slides: Vec::new(),
                current_slide: 0,
                connected_users: HashSet::new(),
            })
        },
    ).await;

    let room_id = RoomId::new("org1", "presentation1");
    let config = RoomConfig {
        room_id: room_id.clone(),
        name: None,
        is_public: true,
        max_users: Some(50),
    };
    
    server.create_room(room_id, "presentation", config).await.unwrap();

    let room_id = RoomId::new("org1", "presentation2");
    let config = RoomConfig {
        room_id: room_id.clone(),
        name: None,
        is_public: true,
        max_users: Some(50),
    };
    

    server.create_room(room_id, "presentation", config).await.unwrap();



    tracing::info!("Server starting on port 5555");
    server.run(5555).await?;

    Ok(())
}
