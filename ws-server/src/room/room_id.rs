use derive_more::Display;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(
    Display, Debug, Clone, Default, Hash, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord,
)]

pub struct RoomId(String);

impl RoomId {
    #[must_use]
    pub fn new() -> Self {
        // Generate a new UUID and prefix it with "room_"
        Self(format!("room_{}", Uuid::new_v4()))
    }

    #[must_use]
    pub fn from_string(id: &str) -> Self {
        Self(id.to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<RoomId> for String {
    fn from(val: RoomId) -> Self {
        val.0
    }
}

impl TryFrom<String> for RoomId {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        // You could add validation here if needed
        Ok(RoomId(s))
    }
}
