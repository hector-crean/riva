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
use room::{presentation::Presentation, room_id::RoomId, RoomMetadata};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    extract::{Data, SocketRef, State}, socket::DisconnectReason, SocketIo
};
use std::{any::{Any, TypeId}, collections::{HashMap, HashSet}, future::Future, net::SocketAddr};
use tokio::sync::RwLock;
use tracing::{info, error, warn, debug, trace};
use ts_rs::TS;
use crate::room::RoomLike;
use std::sync::Arc;




// Define the server state to be shared across handlers

#[derive(Default)]
pub struct ServerState {
    pub rooms: HashMap<RoomId, Box<dyn RoomLike<Command = Box<dyn Any>, Event = Box<dyn Any>>>>,
    command_handlers: HashMap<TypeId, Vec<TypeId>>,
}

impl ServerState {

    pub fn register_room_type<R: RoomLike + 'static, C: 'static>(&mut self) {
        let command_type_id = TypeId::of::<C>();
        let room_type_id = TypeId::of::<R>();
        
        self.command_handlers
            .entry(command_type_id)
            .or_default()
            .push(room_type_id);
    }
    
    pub fn dispatch_command(&mut self, room_id: &RoomId, command: Box<dyn Any>, socket: &SocketRef) -> Option<Box<dyn Any>> {
        if let Some(room) = self.rooms.get_mut(room_id) {
            if room.can_handle_command((*command).type_id()) {
                return room.process_any_command(command, socket);
            }
        }
        None
    }

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



    pub async fn state(&self) -> &RwLock<ServerState> {
        &self.state
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

        socket.on_disconnect(|socket: SocketRef, reason: DisconnectReason| async move {
            info!(
                socket_id = %socket.id,
                namespace = %socket.ns(),
                reason = ?reason,
                "Socket disconnected"
            );
        });

        socket.on("message", || async move {
          info!("Received message");
        });

        socket.on(
            "message",
            |socket: SocketRef,
             Data::<Command>(msg),
             State(state): State<Arc<RwLock<ServerState>>>| async move {
                let room_id = msg.room_id.clone();
                let room_id_str: String = room_id.clone().into();
                
                debug!(
                    socket_id = %socket.id,
                    room_id = %room_id_str,
                    command_type = ?msg.payload,
                    "Received command"
                );
                
                let mut state_guard = state.write().await;

                let event = match state_guard.rooms.get_mut(&room_id) {
                    Some(room) => room.process_command(msg.payload, &socket),
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
                        if let Err(err) = socket.within(room_id_str).emit("message", &response).await {
                            // error!(
                            //     socket_id = %socket.id,
                            //     room_id = %room_id_str.clone(),
                            //     error = %err,
                            //     "Failed to emit event"
                            // );
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

        // Start background task for cleaning up empty rooms
        let server_state = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                let removed = server_state.cleanup_empty_rooms().await;
                if removed > 0 {
                    info!(removed = removed, "Cleaned up empty rooms");
                }
            }
        });

        if let Err(e) = axum::serve(listener, app).await {
            error!(error = %e, "Server error");
            return Err(e.into());
        }

        Ok(())
    }

    // Create a new room
    pub async fn create_room(&self, room_id: RoomId, room_type: &str, config: room::RoomConfig) 
        -> Result<(), String> 
    {
        let mut state = self.state.write().await;
        
        // Check if room already exists
        if state.rooms.contains_key(&room_id) {
            return Err("Room already exists".to_string());
        }
        
        // Create the room
        let room = room::RoomFactory::create_room(room_type, config)?;
        
        // Insert the room
        state.rooms.insert(room_id, room);
        
        Ok(())
    }
    
    // Get room by ID
    pub async fn get_room(&self, room_id: &RoomId) -> Option<RoomMetadata> {
        let state = self.state.read().await;
        state.rooms.get(room_id).map(|room| room.get_metadata())
    }
    
    // List all rooms
    pub async fn list_rooms(&self) -> Vec<(RoomId, RoomMetadata)> {
        let state = self.state.read().await;
        state.rooms
            .iter()
            .map(|(id, room)| (id.clone(), room.get_metadata()))
            .collect()
    }
    
    // Clean up empty rooms
    pub async fn cleanup_empty_rooms(&self) -> usize {
        let mut state = self.state.write().await;
        let empty_rooms: Vec<RoomId> = state.rooms
            .iter()
            .filter(|(_, room)| room.is_empty())
            .map(|(id, _)| id.clone())
            .collect();
        
        let count = empty_rooms.len();
        for room_id in empty_rooms {
            state.rooms.remove(&room_id);
        }
        
        count
    }
}
