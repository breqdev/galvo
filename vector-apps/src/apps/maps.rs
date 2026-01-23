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
        let mut prev: Option<(u8, u8)> = None;

        // Max step distance for interpolation (DAC units)
        const STEP: f32 = 1.0;

        for line in data.lines() {
            let line = line.trim();
            if line.is_empty() {
                // New polyline: insert pen-up or pause if needed
                first_point = true;
                prev = None;
                continue;
            }

            let mut iter = line.split_whitespace();
            let x_norm: f32 = iter.next().unwrap().parse().unwrap();
            let y_norm: f32 = iter.next().unwrap().parse().unwrap();

            // Map normalized [-1,1] to u8 DAC range 0..255
            let x = libm::roundf((x_norm + 1.0) * 0.5 * 255.0) as u8;
            let y = libm::roundf((y_norm * -1.0 + 1.0) * 0.5 * 255.0) as u8;

            if first_point {
                // First point of a polyline
                let delay = match prev {
                    Some((px, py)) => {
                        let dx = x as f32 - px as f32;
                        let dy = y as f32 - py as f32;
                        let distance = libm::sqrtf(dx * dx + dy * dy);

                        (distance * 1.0) as u16
                    }
                    None => {
                        1000 // should be enough
                    }
                };

                path.push(Point {
                    x,
                    y,
                    color: 0,
                    delay,
                });
                path.push(Point {
                    x,
                    y,
                    color: 1,
                    delay: 1,
                });
                prev = Some((x, y));
                first_point = false;
                continue;
            }

            // Interpolate between prev and current
            let (px, py) = prev.unwrap();
            let dx = x as f32 - px as f32;
            let dy = y as f32 - py as f32;
            let distance = libm::sqrtf(dx * dx + dy * dy);
            let steps = libm::ceilf(distance / STEP) as u16;

            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let ix = libm::roundf(px as f32 + dx * t) as u8;
                let iy = libm::roundf(py as f32 + dy * t) as u8;

                path.push(Point {
                    x: ix,
                    y: iy,
                    color: 1,
                    delay: 1, // small delay between interpolated points
                });
            }

            // settle at the last position a little longer
            path.push(Point {
                x,
                y,
                color: 1,
                delay: 100,
            });

            prev = Some((x, y));
        }

        Self { path }
    }
}
impl VectorApp for Maps {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.path
    }
}
