use crate::point::{COLOR_RED, Point};
use hershey_text::{HersheyRenderer, load_mapping};

use alloc::vec::Vec;

fn map_to_dac(v: f32) -> u8 {
    v.clamp(0.0, 255.0) as u8
}

pub fn text_to_path(
    text: &str,
    x: u8,
    y: u8,
    x_scale: f32,
    y_scale: f32,
    font: &'static str,
) -> Vec<Point> {
    let renderer = HersheyRenderer::new();

    let mapping = load_mapping(font).unwrap();

    let strokes: Vec<_> = renderer.render_text(text, mapping);
    let mut points = Vec::with_capacity(strokes.len());

    if let Some(first) = strokes.first() {
        points.push(Point {
            x: map_to_dac((first.x as f32 * x_scale) + x as f32),
            y: map_to_dac((first.y as f32 * y_scale) + y as f32),
            color: if first.pen { COLOR_RED } else { 0 },
            delay: 500,
        });
    }

    for i in 1..strokes.len() {
        let from = &strokes[i - 1];
        let to = &strokes[i];
        let distance = libm::sqrtf(
            libm::powf((from.x as f32 - to.x as f32) * x_scale, 2.0)
                + libm::powf((from.y as f32 - to.y as f32) * y_scale, 2.0),
        );

        let extra_time = if to.pen && !from.pen { 25 } else { 0 };

        points.push(Point {
            x: map_to_dac((to.x as f32 * x_scale) + x as f32),
            y: map_to_dac((to.y as f32 * y_scale) + y as f32),
            color: if to.pen { COLOR_RED } else { 0 },
            delay: ((distance * 15.0) - 2.0).clamp(0.0, 500.0) as u16 + extra_time,
            // delay: (distance * 15.0) as u16 + extra_time,
        });
    }

    points
}
