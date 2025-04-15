pub mod error;
pub mod handlers;
pub mod room;

use axum::routing::get;
use error::WsServerError;

use http;
use room::{presentation::PresentationRoom, room_id::RoomId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
    socket::DisconnectReason,
};
use std::{pin::Pin, sync::Arc};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    future::Future,
    net::SocketAddr,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};
use ts_rs::TS;

// Define the server state to be shared across handlers


pub trait UnTypedRoom: Send + Sync + 'static {
    fn room_type(&self) -> &'static str;
    fn is_empty(&self) -> bool;

    // Type-erased command processing methods
    fn can_handle_command(&self, command_type_id: &TypeId) -> bool;
    fn process_any_command(
        &mut self,
        command: Box<dyn Any>,
        socket: &SocketRef,
    ) -> Option<Box<dyn Any>>;

    // JSON-based command processing for external interfaces
    fn can_handle_command_name(&self, command_name: &str) -> bool;

    // Add these methods for downcasting
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    
    // Process a command from JSON
    fn process_json_command(
        &mut self,
        command_name: &str,
        payload: Value,
        socket: &SocketRef,
    ) -> Result<Option<Value>, String> {
        Err("Not implemented".to_string())
    }
}

// 1. Define a type-safe event system for rooms
pub trait RoomEvent: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    fn event_name(&self) -> &'static str;
}

pub trait RoomCommand: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    const COMMAND_NAME: &'static str;
    fn room_id(&self) -> RoomId;
}

pub trait TypedRoom: UnTypedRoom {
    type Command: RoomCommand;
    type Event: RoomEvent;

    fn room_id(&self) -> RoomId;
    fn process_command(
        &mut self,
        command: Self::Command,
        socket: &SocketRef,
    ) -> Option<Self::Event>;
    fn emit_event(
        &self,
        socket: &SocketRef,
        event: Self::Event,
    ) -> impl Future<Output = Result<(), String>> + Send;
}


#[derive(Default)]
pub struct ServerState {
    pub rooms: HashMap<RoomId, Box<dyn UnTypedRoom>>,
}

impl ServerState {
   

   
    // Clean up empty rooms
    pub async fn cleanup_empty_rooms(&mut self) -> usize {
        let empty_rooms: Vec<RoomId> = self
            .rooms
            .iter()
            .filter(|(_, room)| room.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        let count = empty_rooms.len();
        for room_id in empty_rooms {
            self.rooms.remove(&room_id);
        }

        count
    }

    // Add a typed room to the server state
    pub fn add_room<R: TypedRoom>(&mut self, room: R) {
        let room_id = room.room_id();
        self.rooms.insert(room_id, Box::new(room));
    }
    
    // Get a typed room reference
    pub fn get_room<R: TypedRoom>(&self, room_id: &RoomId) -> Option<&R> {
        self.rooms.get(room_id).and_then(|room| {
            room.as_any().downcast_ref::<R>()
        })
    }
    
    // Get a mutable typed room reference
    pub fn get_room_mut<R: TypedRoom>(&mut self, room_id: &RoomId) -> Option<&mut R> {
        self.rooms.get_mut(room_id).and_then(|room| {
            room.as_any_mut().downcast_mut::<R>()
        })
    }
}

impl<T: TypedRoom> UnTypedRoom for T {
    fn room_type(&self) -> &'static str {
        std::any::type_name::<T>()
    }
    
    fn is_empty(&self) -> bool {
        // This should be implemented by each TypedRoom implementation
        // Default implementation could be overridden by specific room types
        false
    }

    fn can_handle_command(&self, command_type_id: &TypeId) -> bool {
        *command_type_id == TypeId::of::<T::Command>()
    }

    fn process_any_command(
        &mut self,
        command: Box<dyn Any>,
        socket: &SocketRef,
    ) -> Option<Box<dyn Any>> {
        if let Ok(typed_command) = command.downcast::<T::Command>() {
            if let Some(event) = self.process_command(*typed_command, socket) {
                return Some(Box::new(event));
            }
        }
        None
    }

    fn can_handle_command_name(&self, command_name: &str) -> bool {
        command_name == T::Command::COMMAND_NAME
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    
    fn process_json_command(
        &mut self,
        command_name: &str,
        payload: Value,
        socket: &SocketRef,
    ) -> Result<Option<Value>, String> {
        if command_name != T::Command::COMMAND_NAME {
            return Err(format!("Cannot handle command: {}", command_name));
        }
        
        let command: T::Command = serde_json::from_value(payload)
            .map_err(|e| format!("Failed to deserialize command: {}", e))?;
            
        Ok(self.process_command(command, socket)
            .map(|event| serde_json::to_value(event)
                .map_err(|e| format!("Failed to serialize event: {}", e))
                .unwrap_or(Value::Null)))
    }
}

// 4. Improved WsServer with the registry
#[derive(Clone)]
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

    async fn on_connect(socket: SocketRef, State(state): State<Arc<RwLock<ServerState>>>) {
        socket.on("message", |s: SocketRef, Data::<Value>(data)| async move {
            s.broadcast().emit("drawing", &data).await.unwrap();
        });
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
            .route(
                "/room",
                get(handlers::room::get_rooms),
            )
            .with_state(shared_state) // Use the same shared state for route handlers
            .layer(socket_io_layer)
            .layer(cors);
        debug!("Axum router configured");

        info!(port = port, "Binding TCP listener");
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => {
                info!(address = %addr, "TCP listener bound successfully");
                l
            }
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
