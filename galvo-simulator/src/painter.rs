use std::{sync::mpsc::Sender, thread, time::Duration};

use egui::Pos2;

use hershey_text::{HersheyRenderer, fonts, load_mapping};

use crate::point::Point;

const VERTS: [[f32; 3]; 8] = [
    [-0.5, -0.5, -0.5],
    [0.5, -0.5, -0.5],
    [0.5, 0.5, -0.5],
    [-0.5, 0.5, -0.5],
    [-0.5, -0.5, 0.5],
    [0.5, -0.5, 0.5],
    [0.5, 0.5, 0.5],
    [-0.5, 0.5, 0.5],
];

// Edges
const EDGES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

pub fn painter(tx: Sender<Point>) {
    let mut frame = 0;

    let renderer = HersheyRenderer::new();
    let mapping = load_mapping(fonts::ROMANS).unwrap();

    loop {
        frame += 1;

        let angle_x = frame as f32 * 0.02;
        let angle_y = frame as f32 * 0.03;

        let mut points = Vec::new();

        for [i0, i1] in EDGES {
            let v0 = VERTS[i0];
            let v1 = VERTS[i1];

            // Unpack vertices
            let (x0, y0, z0) = (v0[0], v0[1], v0[2]);
            let (x1, y1, z1) = (v1[0], v1[1], v1[2]);

            let (cx, sx) = (angle_x.cos(), angle_x.sin());
            let (cy, sy) = (angle_y.cos(), angle_y.sin());

            // --- Rotate v0 ---
            let y0r = y0 * cx - z0 * sx;
            let z0r = y0 * sx + z0 * cx;
            let x0r = x0 * cy + z0r * sy;
            let z0r2 = -x0 * sy + z0r * cy;

            // --- Rotate v1 ---
            let y1r = y1 * cx - z1 * sx;
            let z1r = y1 * sx + z1 * cx;
            let x1r = x1 * cy + z1r * sy;
            let z1r2 = -x1 * sy + z1r * cy;

            // Perspective projection
            let scale0 = 2.0 / (2.0 + z0r2);
            let px0 = x0r * scale0;
            let py0 = y0r * scale0;

            let scale1 = 2.0 / (2.0 + z1r2);
            let px1 = x1r * scale1;
            let py1 = y1r * scale1;

            points.push(Point {
                x: px0,
                y: py0,
                pen: false,
            });

            // Draw line with stepping
            let steps = 20;
            for s in 0..=steps {
                let t = s as f32 / steps as f32;

                // Interpolate
                let xi = px0 as f32 + t * (px1 as f32 - px0 as f32);
                let yi = py0 as f32 + t * (py1 as f32 - py0 as f32);

                points.push(Point {
                    x: xi,
                    y: yi,
                    pen: true,
                });
            }

            points.push(Point {
                x: px1,
                y: py1,
                pen: true,
            });
        }

        for item in renderer.render_text("hello", mapping) {
            points.push(Point {
                x: item.x as f32 / 100.0 + 0.4,
                y: item.y as f32 / 100.0 + 0.8,
                pen: item.pen,
            })
        }

        for point in points {
            tx.send(point).unwrap();
            thread::sleep(Duration::from_micros(50));
        }
    }
}
