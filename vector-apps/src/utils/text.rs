use crate::point::Point;
use hershey_text::{FontMapping, render_text};

use alloc::vec::Vec;

fn map_to_dac(v: f32) -> u8 {
    v.clamp(0.0, 255.0) as u8
}

const DRAW_VELOCITY: f32 = 20000.0; // galvo units / second
const TRAVEL_VELOCITY: f32 = 40000.0; // faster when laser off
const MAX_STEP: f32 = 4.0; // max distance per emitted point
const DT_MIN: u16 = 0; // µs
const DT_MAX: u16 = 500; // µs
const CORNER_DWELL_US: u16 = 10; // µs at sharp corners

fn distance(a: &hershey_text::Point, b: &hershey_text::Point, xs: f32, ys: f32) -> f32 {
    let dx = (b.x as f32 - a.x as f32) * xs;
    let dy = (b.y as f32 - a.y as f32) * ys;
    libm::sqrtf(dx * dx + dy * dy)
}

fn clamp_dt(dt: f32) -> u16 {
    dt.clamp(DT_MIN as f32, DT_MAX as f32) as u16
}

pub fn text_to_path(
    text: &str,
    x: u8,
    y: u8,
    x_scale: f32,
    y_scale: f32,
    color: u8,
    font: FontMapping,
) -> Vec<Point> {
    let strokes = render_text(text, font);
    let mut points = Vec::new();

    if strokes.is_empty() {
        return points;
    }

    // Emit first point
    let first = &strokes[0];
    points.push(Point {
        x: map_to_dac(first.x as f32 * x_scale + x as f32),
        y: map_to_dac(first.y as f32 * y_scale + y as f32),
        color: if first.pen { color } else { 0 },
        delay: 300,
    });

    for i in 1..strokes.len() {
        let from = &strokes[i - 1];
        let to = &strokes[i];

        let d = distance(from, to, x_scale, y_scale);
        if d == 0.0 {
            continue;
        }

        let velocity = if to.pen {
            DRAW_VELOCITY
        } else {
            TRAVEL_VELOCITY
        };

        // Subdivide long segments
        let steps = libm::ceilf(d / MAX_STEP) as usize;

        for s in 1..=steps {
            let t = s as f32 / steps as f32;

            let fx = from.x as f32 + (to.x as f32 - from.x as f32) * t;
            let fy = from.y as f32 + (to.y as f32 - from.y as f32) * t;

            let step_dist = d / steps as f32;
            let dt = clamp_dt((step_dist / velocity) * 1_000_000.0);

            points.push(Point {
                x: map_to_dac(fx * x_scale + x as f32),
                y: map_to_dac(fy * y_scale + y as f32),
                color: if to.pen { color } else { 0 },
                delay: dt,
            });
        }

        // ---- Corner dwell (cheap version) ----
        if i + 1 < strokes.len() && from.pen && to.pen {
            let next = &strokes[i + 1];

            let ax = from.x as f32 - to.x as f32;
            let ay = from.y as f32 - to.y as f32;
            let bx = next.x as f32 - to.x as f32;
            let by = next.y as f32 - to.y as f32;

            let adotb = ax * bx + ay * by;
            let amag = libm::sqrtf(ax * ax + ay * ay);
            let bmag = libm::sqrtf(bx * bx + by * by);

            if amag > 0.0 && bmag > 0.0 {
                let cos_theta = (adotb / (amag * bmag)).clamp(-1.0, 1.0);
                let sharpness = 1.0 - cos_theta;

                if sharpness > 0.3 {
                    points.push(Point {
                        x: map_to_dac(to.x as f32 * x_scale + x as f32),
                        y: map_to_dac(to.y as f32 * y_scale + y as f32),
                        color,
                        delay: (CORNER_DWELL_US as f32 * sharpness) as u16,
                    });
                }
            }
        }
    }

    points
}
