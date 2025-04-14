use socketioxide::extract::SocketRef;

pub mod presentation;
pub mod room_id;


pub trait RoomLike: Send + Sync {
    type Command;
    type Event;
    fn transaction(&mut self, command: Self::Command, socket: &SocketRef) -> Option<Self::Event>;
    fn add_client(&mut self, client_id: String, socket_id: String) -> bool;
    fn remove_client(&mut self, client_id: &str) -> bool;
    fn is_empty(&self) -> bool;
}
