#![no_std]

extern crate alloc;

use alloc::vec::Vec;

#[derive(Debug, Copy, Clone)]
struct PackedPoint {
    pub x: i8,
    pub y: i8,
    pub pen: bool,
}

#[derive(Debug, Copy, Clone)]
struct Glyph {
    pub left: i8,
    pub right: i8,
    pub strokes: &'static [PackedPoint],
}

pub struct Point {
    pub x: i16,
    pub y: i16,
    pub pen: bool,
}

impl Default for Point {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            pen: false,
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/chr_font.rs"));

pub fn render_text(text: &str) -> Vec<Point> {
    let mut result = Vec::new();
    let mut x_idx = 0;

    for character in text.chars() {
        if let Some(glyph) = CHR_FONT[character as usize] {
            result.extend(glyph.strokes.iter().map(|point| Point {
                x: point.x as i16 - glyph.left as i16 + x_idx,
                y: point.y as i16,
                pen: point.pen,
            }));
            x_idx += glyph.right as i16 - glyph.left as i16;
        }
    }

    result
}
