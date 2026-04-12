use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    GetTheme,
    SetVolume { level: u8 },
    GetVolume,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Ok,
    Volume { level: u8 },
    Error { message: String },
}
