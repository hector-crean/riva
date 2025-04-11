pub mod error;
pub mod events;

use axum::routing::get;
use error::WsServerError;
use events::{
   JoinRoom, JoinRoomMessage, PresentationRoomMessage
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

        socket.on(
            "presentation_message",
            |socket: SocketRef,
             Data::<PresentationRoomMessage>(msg),
             State(state): State<Arc<RwLock<ServerState>>>| async move {
                match msg {
                    PresentationRoomMessage::JoinRoom(msg) => { }
                    PresentationRoomMessage::LeaveRoom(msg) => {
                    }
                    PresentationRoomMessage::RequestSlideChange(msg) => {
                    }
                    PresentationRoomMessage::RoomJoined(msg) => {
                    }
                    PresentationRoomMessage::RoomLeft(msg) => {
                    }
                    PresentationRoomMessage::SlideChanged(msg) => {
                    }
                }
             
            },
        );

      

       
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
