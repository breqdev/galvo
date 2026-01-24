use alloc::vec::Vec;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: u8,
    pub y: u8,
    pub color: (u8, u8, u8),
    pub delay: u16,
}

pub type Path = Vec<Point>;
