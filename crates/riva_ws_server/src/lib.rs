pub mod error;
pub mod events;

use axum::routing::get;
use error::WsServerError;
use events::{
    Drawing, DrawingPayload, RivaWsMessage, ClientJoinRoom, JoinRoom, JoinRoomPayload, ClientMessage,
    LeaveRoom, LeaveRoomPayload, SlideChange, SlideChangePayload, SlideData, SlideDataPayload,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use socketioxide::{
    SocketIo,
    extract::{AckSender, Data, SocketRef, State},
};
use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    net::SocketAddr,
};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use ts_rs::TS;

use std::sync::{Arc, Mutex};
use std::time::Duration;




#[derive(Debug, Clone)]
pub enum Room {
    Presentation {
        name: String,
        current_slide: usize,
        slide_data: Vec<Value>,
        clients: Vec<String>, // Socket IDs 
    }
}

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq, Serialize, Deserialize, TS, PartialOrd, Ord)]
#[ts(export)]
pub struct RoomId {
    room_name: String,
    organisation_id: String,
}

impl RoomId {
    pub fn new(organisation_id: &str, room_name: &str) -> Self {
        Self { 
            organisation_id: organisation_id.to_string(), 
            room_name: room_name.to_string() 
        }
    }
}


impl Into<String> for RoomId {
    fn into(self) -> String {
        format!("{}:{}", self.organisation_id, self.room_name)
    }
}

impl TryFrom<String> for RoomId {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.split_once(':') {
            Some((org_id, room_name)) => Ok(RoomId {
                organisation_id: org_id.to_string(),
                room_name: room_name.to_string(),
            }),
            None => Err("Invalid RoomId format, expected 'organisation_id:room_name'"),
        }
    }
}

// Define the server state to be shared across handlers

#[derive(Default, Clone)]
pub struct ServerState {
    pub rooms: HashMap<RoomId, Room>,
    pub messages: HashMap<String, VecDeque<RivaWsMessage>>
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

    async fn on_connect(socket: SocketRef) {
        info!("socket connected: {}", socket.id);

        // Handle join room events
        socket.on(
            JoinRoom::EVENT_NAME,
            |socket: SocketRef,
             Data::<ClientJoinRoom>(msg),
             state: State<Arc<RwLock<ServerState>>>| async move {
                socket.leave_all();
                let room_id = msg.payload.room_id;
                socket.join(Into::<String>::into( room_id.clone()));
                
                // Update server state to track client in room
                let mut server_state = state.write().await;
                
                // Create room if it doesn't exist
                if !server_state.rooms.contains_key(&room_id) {
                    server_state.rooms.insert(
                        room_id.clone(),
                        Room::Presentation {
                            name: room_id.room_name.clone(),
                            current_slide: 0,
                            slide_data: Vec::new(),
                            clients: vec![socket.id.to_string()],
                        },
                    );
                }
                
                // Broadcast join event to room
                socket.to(Into::<String>::into(room_id.clone())).emit(JoinRoom::EVENT_NAME, &room_id).await;
                
                // Send current room state to the joining client
                if let Some(Room::Presentation { current_slide, slide_data, .. }) = 
                    server_state.rooms.get(&room_id) {
                    // Send current slide information
                   
                }
            },
        );

      

        // Handle disconnect
        socket.on_disconnect(|socket: SocketRef, state: State<Arc<RwLock<ServerState>>>| async move {
            info!("socket disconnected: {}", socket.id);
            
            // Clean up client from all rooms
            let mut server_state = state.write().await;
           
        });
    }

    pub async fn run(&self, port: u16) -> Result<(), WsServerError> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let (socket_io_layer, io) = SocketIo::builder()
            .with_state(self.state.clone())
            .build_layer();

        // Register the on_connect handler for the default namespace
        io.ns("/", Self::on_connect);

        let app = axum::Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .with_state(io)
            .layer(socket_io_layer);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        info!("Server running on http://{}", addr);
        let _ = axum::serve(listener, app).await;

        Ok(())
    }
}
