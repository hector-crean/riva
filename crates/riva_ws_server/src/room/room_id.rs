use serde::{Deserialize, Serialize};
use ts_rs::TS;


#[derive(
    Debug, Clone, Default, Hash, Eq, PartialEq, Serialize, Deserialize, TS, PartialOrd, Ord,
)]
#[ts(export)]
pub struct RoomId {
    room_name: String,
    organisation_id: String,
}

impl RoomId {
    pub fn new(organisation_id: &str, room_name: &str) -> Self {
        Self {
            organisation_id: organisation_id.to_string(),
            room_name: room_name.to_string(),
        }
    }
}

impl From<RoomId> for String {
    fn from(val: RoomId) -> Self {
        format!("{}:{}", val.organisation_id, val.room_name)
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
