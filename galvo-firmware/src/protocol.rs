use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    SetIndicatorLight { r: u8, g: u8, b: u8 },
}

#[derive(Serialize)]
pub struct Response {
    pub success: bool,
}
