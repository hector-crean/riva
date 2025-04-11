use axum::routing::get;
use serde_json::Value;
use socketioxide::{
    SocketIo,
    extract::{AckSender, Data, SocketRef},
};
use std::{future::Future, net::SocketAddr};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use tracing_subscriber::FmtSubscriber;
use strum::{EnumString, Display, AsRefStr, IntoStaticStr};
use ts_rs::TS;

use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum WsServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Subscriber(#[from] tracing::subscriber::SetGlobalDefaultError),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomTag;




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinRoomEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveRoomEvent;


// Define payload types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JoinRoomPayload {
    room: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaveRoomPayload {
    room: String,
}



// Define a unified message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<E, T> {
    #[serde(rename = "type")]
    pub event_type: E,
    #[serde(flatten)]
    pub payload: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,
}




pub type JoinRoom = Message<JoinRoomEvent, JoinRoomPayload>;
pub type LeaveRoom = Message<LeaveRoomEvent, LeaveRoomPayload>;



// Replace the individual message type aliases with an enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Event {
    JoinRoom(JoinRoom ),
    LeaveRoom(LeaveRoom),
}



mod handlers {
    use super::*;

    pub fn on_connect(socket: SocketRef, Data(data): Data<Value>) {
        info!("Socket.IO connected: {:?} {:?}", socket.ns(), socket.id);
        
        // Set up event handlers
        socket.on("JoinRoom", handle_join_room);
        socket.on("LeaveRoom", handle_leave_room);

    }
    
    async fn handle_join_room(socket: SocketRef, Data(Message { event_type, payload, id, in_reply_to }): Data<JoinRoom>, ack: AckSender) {
        let room = payload.room.clone();
        info!("Join room request: {:?}", room);

       
        
    }
    
    async fn handle_leave_room(socket: SocketRef, Data(Message { event_type, payload, id, in_reply_to }): Data<LeaveRoom>, ack: AckSender) {
       
    }
    


}

pub struct WsServer {
}

impl WsServer {
    pub fn new() -> Self {
       
        Self {  }
    }
   

    pub async fn run(&self, port: u16) -> Result<(), WsServerError> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));

        let (socket_io_layer, io) = SocketIo::new_layer();

        io.ns("/", handlers::on_connect);
        io.ns("/custom", handlers::on_connect);


        let app = axum::Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .layer(socket_io_layer);

        let listener = tokio::net::TcpListener::bind(addr).await?;

        info!("Server running on http://{}", addr);
        let _ = axum::serve(listener, app).await;

        Ok(())
    }
}




