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

pub type FontMapping = [u16; 256];

include!(concat!(env!("OUT_DIR"), "/hershey_font.rs"));

pub fn render_text(text: &str, mapping: FontMapping) -> Vec<Point> {
    let mut result = Vec::new();
    let mut x_idx = 0;

    for character in text.chars() {
        if character > 255 as char {
            continue;
        }

        let hershey_id = mapping[character as usize] as usize;

        if hershey_id == 0 || hershey_id >= HERSHEY_FONT.len() {
            continue;
        }

        if let Some(glyph) = HERSHEY_FONT[hershey_id] {
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
