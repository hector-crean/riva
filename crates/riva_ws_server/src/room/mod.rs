use presentation::Presentation;
use socketioxide::extract::SocketRef;
use crate::events::{CommandType, EventType};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use std::any::{Any, TypeId};


pub mod presentation;
pub mod video;
pub mod room_id;

pub trait RoomLike: Send + Sync {
    type Command;
    type Event;

    fn process_any_command(&mut self, command: Box<dyn Any>, socket: &SocketRef) -> Option<Box<dyn Any>>;
    fn can_handle_command(&self, command_type_id: TypeId) -> bool;

    
    // Type-safe internal implementation
    fn process_typed_command(&mut self, command: Self::Command, socket: &SocketRef) -> Option<Self::Event>;
    

    // Process a command and return an event if needed
    fn process_command(&mut self, command: CommandType, socket: &SocketRef) -> Option<EventType>;
    
    // Handle user joining the room
    fn join_user(&mut self, socket_id: String) -> Option<EventType>;
    
    // Handle user leaving the room
    fn leave_user(&mut self, socket_id: String) -> Option<EventType>;
    
    // Get room metadata
    fn get_metadata(&self) -> RoomMetadata;
    
    // Get room type as string
    fn room_type(&self) -> &'static str;
    
    // Get active users in the room
    fn get_users(&self) -> Vec<String>;
    
    // Check if room is empty
    fn is_empty(&self) -> bool {
        self.get_users().is_empty()
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct RoomMetadata {
    pub room_type: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_count: usize,
    // Add more common metadata fields
}

pub struct RoomFactory;

impl RoomFactory {
    pub fn create_room(room_type: &str, config: RoomConfig) -> Result<Box<dyn RoomLike>, String> {
        match room_type {
            "presentation" => Ok(Box::new(Presentation::new(
                config.name.unwrap_or_else(|| "Untitled Presentation".to_string()),
                config.initial_slide.unwrap_or(0),
                config.slide_data.unwrap_or_default(),
            ))),
            // Add other room types here
            _ => Err(format!("Unsupported room type: {}", room_type)),
        }
    }
}

#[derive(Default)]
pub struct RoomConfig {
    pub name: Option<String>,
    pub initial_slide: Option<usize>,
    pub slide_data: Option<Vec<serde_json::Value>>,
    // Add other configuration options as needed
}
