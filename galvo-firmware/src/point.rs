use serde::Deserialize;

#[derive(Copy, Clone, Deserialize)]
pub struct Point {
    pub x: u8,
    pub y: u8,
    pub delay: u16,
    pub color: u8,
}

pub const COLOR_RED: u8 = 0b0001;
pub const COLOR_GREEN: u8 = 0b0010;
pub const COLOR_BLUE: u8 = 0b0100;
