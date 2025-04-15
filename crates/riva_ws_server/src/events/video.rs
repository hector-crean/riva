
use serde::{Deserialize, Serialize};
use ts_rs::TS;




#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum VideoCommand {
   StartVideo
}





#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(tag = "type")]
pub enum VideoEvent {
    VideoStarted
}
