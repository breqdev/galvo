use alloc::vec::Vec;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
};

/// The `RoadsVectorApp` struct loads a text file of normalized polylines
/// (one line per point: x y in [-1,1], empty line = pen-up / new polyline),
/// scales to u8 DAC coordinates (0..255),
/// and precomputes a flat Path.
pub struct Maps {
    path: Path,
}

impl Maps {
    pub fn new() -> Self {
        let mut path: Path = Vec::new();

        // Include the file in the binary at compile time
        let data = include_str!("roads.txt");

        let mut first_point = true;

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                // optional: insert pen-up / pause if needed
                first_point = true;
                continue;
            }

            let mut iter = line.split_whitespace();
            let x_norm: f32 = iter.next().unwrap().parse().unwrap();
            let y_norm: f32 = iter.next().unwrap().parse().unwrap();

            // Map normalized [-1,1] to u8 DAC range 0..255
            let x = libm::roundf((x_norm + 1.0) * 0.5 * 255.0) as u8;
            let y = libm::roundf((y_norm * -1.0 + 1.0) * 0.5 * 255.0) as u8;

            let point = Point {
                x,
                y,
                color: if first_point { 0 } else { 1 },
                delay: if first_point { 200 } else { 10 }, // longer for first point of polyline
            };

            path.push(point);
            first_point = false;
        }

        Self { path }
    }
}

impl VectorApp for Maps {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.path
    }
}
