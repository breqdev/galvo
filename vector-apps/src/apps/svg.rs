use alloc::vec::Vec;
use itertools::Itertools;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
};

pub struct SvgApp {
    points: Vec<Point>,
}

fn coord_to_color(x: u8, y: u8) -> (u8, u8, u8) {
    let top = (255.0, 33.0, 140.0);
    let left = (20.0, 177.0, 255.0);
    let right = (255.0, 218.0, 0.0);

    // let left = (20.0, 117.0, 255.0);
    // let top = (255.0, 255.0, 255.0);
    // let right = (255.0, 33.0, 140.0);

    let cx = 127.5_f32;
    let cy = 127.5_f32;
    let radius = 100.0_f32;

    let pi = core::f32::consts::PI;

    let anchors_pos = [
        (
            cx + radius * libm::cosf(270.0 * pi / 180.0),
            cy + radius * libm::sinf(270.0 * pi / 180.0),
        ),
        (
            cx + radius * libm::cosf(30.0 * pi / 180.0),
            cy + radius * libm::sinf(30.0 * pi / 180.0),
        ),
        (
            cx + radius * libm::cosf(150.0 * pi / 180.0),
            cy + radius * libm::sinf(150.0 * pi / 180.0),
        ),
    ];
    let colors = [top, left, right];

    let px = x as f32;
    let py = y as f32;

    let eps = 1e-6_f32;
    let mut weights = [0.0_f32; 3];
    for (i, (ax, ay)) in anchors_pos.iter().enumerate() {
        let dist = libm::sqrtf((px - ax) * (px - ax) + (py - ay) * (py - ay));
        weights[i] = 1.0 / if dist < eps { eps } else { dist };
    }

    let total: f32 = weights[0] + weights[1] + weights[2];

    let r =
        (weights[0] * colors[0].0 + weights[1] * colors[1].0 + weights[2] * colors[2].0) / total;
    let g =
        (weights[0] * colors[0].1 + weights[1] * colors[1].1 + weights[2] * colors[2].1) / total;
    let b =
        (weights[0] * colors[0].2 + weights[1] * colors[1].2 + weights[2] * colors[2].2) / total;

    (r as u8, g as u8, b as u8)
}

impl SvgApp {
    pub fn new() -> Self {
        let points = include_str!("avalogo.txt")
            .lines()
            .scan(None, |last, line| {
                let result = line.split(" ").collect_vec();
                let pen = result[0] == "+";
                let x: u8 = result[1].parse().unwrap();
                let y: u8 = result[2].parse().unwrap();

                let pen_change = pen
                    != last
                        .map(|point: Point| point.color != (0, 0, 0))
                        .unwrap_or(false);

                let new = Point {
                    x,
                    y,
                    color: if pen { coord_to_color(x, y) } else { (0, 0, 0) },
                    delay: if last.is_none() {
                        2000
                    } else if pen_change {
                        if let Some(point) = last {
                            let dx = x.abs_diff(point.x) as f32;
                            let dy = y.abs_diff(point.y) as f32;
                            let d = libm::sqrtf((dx * dx) + (dy * dy));

                            (d * 50.0) as u16
                        } else {
                            2000
                        }
                    } else {
                        20
                    },
                };

                *last = Some(new);

                Some(new)
            })
            .collect();

        Self { points }
    }
}

impl VectorApp for SvgApp {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.points
    }
}
