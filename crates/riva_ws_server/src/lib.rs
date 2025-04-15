pub mod error;
pub mod handlers;
pub mod room;

use axum::routing::get;
use error::WsServerError;

use http;
use room::room_id::RoomId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socketioxide::{
    SocketIo,
    extract::{Data, SocketRef, State},
    socket::DisconnectReason,
};
use std::sync::Arc;
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

#[derive(Default)]
pub struct ServerState {
    pub rooms: HashMap<RoomId, Box<dyn RoomLike>>,
    command_handlers: HashMap<TypeId, Vec<Box<dyn CommandHandler>>>,
}

impl ServerState {
    // Get room by ID
    pub async fn get_room(&self, room_id: &RoomId) -> Option<RoomMetadata> {
        self.rooms.get(room_id).map(|room| room.get_metadata())
    }

    // List all rooms
    pub async fn list_rooms(&self) -> Vec<(RoomId, RoomMetadata)> {
        self.rooms
            .iter()
            .map(|(id, room)| (id.clone(), room.get_metadata()))
            .collect()
    }

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
}

// Define a trait for command handlers to improve type safety
pub trait CommandHandler: Send + Sync {
    fn command_type_id(&self) -> TypeId;
    fn handle(&self, command: Box<dyn Any>, socket: &SocketRef) -> Option<Box<dyn Any>>;
}

// Generic implementation for specific command/event types
struct TypedCommandHandler<C: Send + Sync + 'static, E: Send + Sync + 'static> {
    handler: Box<dyn Fn(C, &SocketRef) -> Option<E> + Send + Sync>,
    _phantom: std::marker::PhantomData<(C, E)>,
}

impl<C: Send + Sync + 'static, E: Send + Sync + 'static> CommandHandler
    for TypedCommandHandler<C, E>
{
    fn command_type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }

    fn handle(&self, command: Box<dyn Any>, socket: &SocketRef) -> Option<Box<dyn Any>> {
        match command.downcast::<C>() {
            Ok(concrete_command) => (self.handler)(*concrete_command, socket)
                .map(|event| Box::new(event) as Box<dyn Any>),
            Err(_) => None,
        }
    }
}

impl ServerState {
    // Register a handler for a specific command type with improved type safety
    pub fn register_command_handler<C: Send + Sync + 'static, E: Send + Sync + 'static>(
        &mut self,
        handler: impl Fn(C, &SocketRef) -> Option<E> + Send + Sync + 'static,
    ) -> &mut Self {
        // Return self for method chaining
        let command_type_id = TypeId::of::<C>();

        let typed_handler = TypedCommandHandler {
            handler: Box::new(handler),
            _phantom: std::marker::PhantomData,
        };

        self.command_handlers
            .entry(command_type_id)
            .or_default()
            .push(Box::new(typed_handler));

        self // Return self for method chaining
    }
}

// A more structured approach for room management with socketioxide

// 1. Define a type-safe event system for rooms
pub trait RoomEvent: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    fn event_name(&self) -> &'static str;
}

pub trait RoomCommand: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    const COMMAND_NAME: &'static str;
    fn room_id(&self) -> RoomId;
}

// 2. Improved RoomLike trait with generic type parameters
pub trait RoomLike: Send + Sync + 'static {
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

    fn get_metadata(&self) -> RoomMetadata;
}

pub trait RoomEmitter: RoomLike {
    fn emit_event(
        &self,
        socket: &SocketRef,
        event: Box<dyn Any + Send + Sync>,
    ) -> impl Future<Output = Result<(), String>> + Send;
}

// Improved TypedRoom trait with better type safety and serialization support
pub trait TypedRoom: RoomLike {
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

impl<T: TypedRoom> RoomLike for T {
    fn room_type(&self) -> &'static str {
        std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap_or("Unknown")
    }

    fn is_empty(&self) -> bool {
        // Implementation depends on your empty state
        true
    }

    fn can_handle_command(&self, command_type_id: &TypeId) -> bool {
        *command_type_id == TypeId::of::<T::Command>()
    }

    fn process_any_command(
        &mut self,
        command: Box<dyn Any>,
        socket: &SocketRef,
    ) -> Option<Box<dyn Any>> {
        if let Ok(concrete_command) = command.downcast::<T::Command>() {
            self.process_command(*concrete_command, socket)
                .map(|event| Box::new(event) as Box<dyn Any>)
        } else {
            None
        }
    }

    // These would now leverage the RoomCommand/RoomEvent traits
    fn can_handle_command_name(&self, command_name: &str) -> bool {
        // Now we can use the COMMAND_NAME constant from RoomCommand
        command_name == T::Command::COMMAND_NAME
    }

    fn get_metadata(&self) -> RoomMetadata {
        // Default implementation that can be overridden
        RoomMetadata {
            room_type: self.room_type().to_string(),
            name: None,
            user_count: 0,
            is_public: true,
            metadata: HashMap::new(),
        }
    }
}

impl<T: TypedRoom> RoomEmitter for T {
    async fn emit_event(
        &self,
        socket: &SocketRef,
        event: Box<dyn Any + Send + Sync>,
    ) -> Result<(), String> {
        if let Ok(concrete_event) = event.downcast::<T::Event>() {
            let room_id_str: String = self.room_id().into();
            socket
                .within(room_id_str)
                .emit(T::Event::event_name(&concrete_event), &*concrete_event)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("Event type mismatch".to_string())
        }
    }
}

// 3. Room registry for managing different room types
pub struct RoomRegistry {
    factories:
        HashMap<String, Box<dyn Fn(RoomConfig) -> Result<Box<dyn RoomLike>, String> + Send + Sync>>,
}

// Define RoomConfig for room creation and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    pub name: Option<String>,
    pub max_users: Option<usize>,
    pub is_public: bool,
    pub metadata: HashMap<String, Value>,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            name: None,
            max_users: None,
            is_public: true,
            metadata: HashMap::new(),
        }
    }
}

// Also need to define RoomMetadata which is used in get_room and list_rooms methods
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RoomMetadata {
    pub room_type: String,
    pub name: Option<String>,
    pub user_count: usize,
    pub is_public: bool,
    pub metadata: HashMap<String, Value>,
}

impl RoomRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register<R, F>(&mut self, room_type: &str, factory: F)
    where
        R: RoomLike + 'static,
        F: Fn(RoomConfig) -> Result<R, String> + Send + Sync + 'static,
    {
        let boxed_factory = Box::new(
            move |config: RoomConfig| -> Result<Box<dyn RoomLike>, String> {
                factory(config).map(|room| Box::new(room) as Box<dyn RoomLike>)
            },
        );

        self.factories.insert(room_type.to_string(), boxed_factory);
    }

    pub fn create_room(
        &self,
        room_type: &str,
        config: RoomConfig,
    ) -> Result<Box<dyn RoomLike>, String> {
        match self.factories.get(room_type) {
            Some(factory) => factory(config),
            None => Err(format!("Unknown room type: {}", room_type)),
        }
    }
}

// 4. Improved WsServer with the registry
#[derive(Clone)]
pub struct WsServer {
    pub state: Arc<RwLock<ServerState>>,
    registry: Arc<RoomRegistry>,
}

impl WsServer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            registry: Arc::new(RoomRegistry::new()),
        }
    }

    pub fn with_registry(registry: RoomRegistry) -> Self {
        Self {
            state: Arc::new(RwLock::new(ServerState::default())),
            registry: Arc::new(registry),
        }
    }

    // Register a room type
    pub fn register_room_type<R, F>(&mut self, room_type: &str, factory: F)
    where
        R: RoomLike + 'static,
        F: Fn(RoomConfig) -> Result<R, String> + Send + Sync + 'static,
    {
        let registry =
            Arc::get_mut(&mut self.registry).expect("Cannot modify registry after sharing");
        registry.register(room_type, factory);
    }

    // Create a new room using the registry
    pub async fn create_room(
        &self,
        room_id: RoomId,
        room_type: &str,
        config: RoomConfig,
    ) -> Result<(), String> {
        let mut state = self.state.write().await;

        // Check if room already exists
        if state.rooms.contains_key(&room_id) {
            return Err("Room already exists".to_string());
        }

        // Create the room using the registry
        let room = self.registry.create_room(room_type, config)?;

        // Insert the room
        state.rooms.insert(room_id, room);

        Ok(())
    }

    // Add a method to register global command handlers
    pub async fn register_command_handler<C: Send + Sync + 'static, E: Send + Sync + 'static>(
        &self,
        handler: impl Fn(C, &SocketRef) -> Option<E> + Send + Sync + 'static,
    ) {
        let mut state = self.state.write().await;
        state.register_command_handler(handler);
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
        // io.ns("/", Self::on_connect);

        debug!("Root namespace handler registered");

        let app = axum::Router::new()
            .route(
                "/room",
                get(handlers::room::get_rooms).post(handlers::room::create_room),
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

        // Start background task for cleaning up empty rooms
        let server_state = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                let removed = server_state.state.write().await.cleanup_empty_rooms().await;
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
}
