pub mod error;
pub mod events;
pub mod handlers;
pub mod room;

use axum::routing::get;
use error::WsServerError;
use events::{
    presentation::{PresentationCommand, PresentationEvent}, Command, CommandType, Event, EventType
};
use http;
use room::{presentation::Presentation, room_id::RoomId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    extract::{Data, SocketRef, State}, socket::DisconnectReason, SocketIo
};
use std::{collections::{HashMap, HashSet}, future::Future, net::SocketAddr};
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug, trace};
use ts_rs::TS;
use crate::room::RoomLike;
use std::sync::Arc;




// Define the server state to be shared across handlers

#[derive(Clone, Debug, TS, Deserialize, Serialize)]
#[ts(export)]
#[serde(tag = "type", content = "payload")]
pub enum Room {
    Presentation(Presentation),
}

impl Room {
    fn add_client(&mut self, socket_id: &str) -> bool {
        match self {
            Room::Presentation(presentation) => presentation.add_client(socket_id),
            // Add cases for future variants here
        }
    }

    fn remove_client(&mut self, socket_id: &str) -> bool {
        match self {
            Room::Presentation(presentation) => presentation.remove_client(socket_id),
            // Add cases for future variants here
        }
    }

    // Implement any other methods required by the RoomLike trait
    // following the same pattern
}





#[derive(Default, Clone)]
pub struct ServerState {
    pub rooms: HashMap<RoomId, Room>,
}

#[derive(Clone, Default)]
pub struct WsServer {
    pub state: Arc<RwLock<ServerState>>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
        }
    }

    pub async fn state(&self) -> ServerState {
        self.state.read().await.clone()
    }

    pub async fn with_state_mut<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ServerState) -> Fut,
        Fut: Future<Output = R>,
    {
        let mut state = self.state.write().await;
        f(&mut state).await
    }

    async fn on_connect(socket: SocketRef, State(_): State<Arc<RwLock<ServerState>>>) {
        info!(socket_id = %socket.id, "Socket connected");

        socket.on_disconnect(|socket: SocketRef, reason: DisconnectReason, State(state): State<Arc<RwLock<ServerState>>>| async move {
            info!(
                socket_id = %socket.id,
                namespace = %socket.ns(),
                reason = ?reason,
                "Socket disconnected"
            );
            
            // Clean up by removing the client from any rooms they were in
            let mut state_guard = state.write().await;
            let socket_id = socket.id.to_string();
            
            // Iterate through all rooms and remove the disconnected client using the RoomLike trait
            for (room_id, room) in state_guard.rooms.iter_mut() {
                let room_id_str: String = room_id.clone().into();
                if room.remove_client(&socket_id) {
                    info!(
                        socket_id = %socket.id,
                        room_id = %room_id_str,
                        "Client removed from room"
                    );
                }
            }
        });

        socket.on(
            "message",
            |socket: SocketRef,
             Data::<Command>(msg),
             State(state): State<Arc<RwLock<ServerState>>>| async move {
                let room_id = msg.room_id.clone();
                let room_id_str: String = room_id.clone().into();
                
                info!(
                    socket_id = %socket.id,
                    room_id = %room_id_str,
                    command_type = ?msg.payload,
                    "Received command"
                );
                
                let mut state_guard = state.write().await;

                let event = match state_guard.rooms.get_mut(&room_id) {
                    Some(room) => match (room, msg.payload) {
                        (Room::Presentation(presentation), CommandType::Presentation(cmd)) => {
                            info!(
                                socket_id = %socket.id,
                                room_id = %room_id_str,
                                command = ?cmd,
                                "Processing presentation command"
                            );
                            presentation
                                .transaction(room_id, cmd, &socket)
                                .map(|event| EventType::Presentation(event))
                        }
                        (_, _) => {
                            warn!(
                                socket_id = %socket.id,
                                room_id = %room_id_str,
                                // command_type = ?cmd_type,
                                "Unsupported operation for room type"
                            );
                            None
                        }
                    },
                    None => {
                        warn!(
                            socket_id = %socket.id,
                            room_id = %room_id_str,
                            "Room not found"
                        );
                        None
                    }
                };

                match event {
                    Some(response) => {
                        debug!(
                            socket_id = %socket.id,
                            room_id = %room_id_str,
                            event_type = ?response,
                            "Emitting event to room"
                        );
                        match socket.within(room_id_str.clone()).emit("message", &response).await {
                            Ok(_) => {
                                info!(
                                    socket_id = %socket.id,
                                    room_id = %room_id_str,
                                    "Event emitted to room"
                                );
                            },
                            Err(err) => {
                                error!(
                                    socket_id = %socket.id,
                                    room_id = %room_id_str,
                                    error = %err,
                                    "Failed to emit event"
                                );
                            }
                        }
                    }
                    None => {
                        trace!(
                            socket_id = %socket.id,
                            room_id = %room_id_str,
                            "No event to emit"
                        );
                    }
                }
            },
        );
    }

    pub async fn run(&self, port: u16) -> Result<(), WsServerError> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!(address = %addr, "Starting WebSocket server");

        // Configure CORS with explicit headers instead of Any
        let cors = tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods([
                http::Method::GET,
                http::Method::POST,
                http::Method::PUT,
                http::Method::DELETE,
                http::Method::OPTIONS,
            ])
            .allow_headers([
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::ORIGIN,
            ]);
        debug!("CORS configuration set up");

        // Create a shared state that will be used by both SocketIO and route handlers
        let shared_state = self.state.clone();

        let (socket_io_layer, io) = SocketIo::builder()
            .with_state(shared_state.clone())
            .build_layer();
        debug!("SocketIO layer created");

        // Register the on_connect handler for the root namespace
        io.ns("/", Self::on_connect);
        debug!("Root namespace handler registered");

        let app = axum::Router::new()
            .route("/room", get(handlers::room::get_rooms).post(handlers::room::create_room))
            .with_state(shared_state) // Use the same shared state for route handlers
            .layer(socket_io_layer)
            .layer(cors);
        debug!("Axum router configured");

        info!(port = port, "Binding TCP listener");
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => {
                info!(address = %addr, "TCP listener bound successfully");
                l
            },
            Err(e) => {
                error!(address = %addr, error = %e, "Failed to bind TCP listener");
                return Err(e.into());
            }
        };

        info!(address = %addr, "Starting server");
        if let Err(e) = axum::serve(listener, app).await {
            error!(error = %e, "Server error");
            return Err(e.into());
        }

        Ok(())
    }
}
