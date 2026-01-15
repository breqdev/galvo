use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::point::Point;

#[derive(Deserialize)]
#[serde(tag = "cmd")]
pub enum Command {
    SetIndicatorLight { r: u8, g: u8, b: u8 },
    SetWaveform { points: Vec<Point> },
}

#[derive(Serialize)]
pub struct Response {
    pub success: bool,
}
