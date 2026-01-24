use crate::{
    point::{Path, Point},
    utils::text::text_to_path,
};
use alloc::vec::Vec;
use hershey_text::fonts;

use crate::apps::VectorApp;

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

fn map_to_dac(v: f32) -> u8 {
    let x = v * 127.0 + 128.0;
    x.clamp(0.0, 255.0) as u8
}

pub struct CubeDemo {
    points: Vec<Point>,
    static_points: Vec<Point>,
}

impl CubeDemo {
    pub fn new() -> Self {
        let mut static_points = Vec::new();

        static_points.append(&mut text_to_path(
            "Hello",
            0,
            16,
            1.0,
            1.0,
            (255, 0, 0),
            fonts::ROMANS,
        ));
        static_points.append(&mut text_to_path(
            "World",
            176,
            240,
            1.0,
            1.0,
            (255, 0, 0),
            fonts::ROMANS,
        ));

        Self {
            points: Vec::new(),
            static_points,
        }
    }
}

impl VectorApp for CubeDemo {
    fn get_path(&mut self, frame: u64) -> &Path {
        let color = (255, 0, 0);

        let angle_x = frame as f32 * 0.02;
        let angle_y = frame as f32 * 0.03;

        self.points.clear();

        for [i0, i1] in EDGES {
            let v0 = VERTS[i0];
            let v1 = VERTS[i1];

            // Unpack vertices
            let (x0, y0, z0) = (v0[0], v0[1], v0[2]);
            let (x1, y1, z1) = (v1[0], v1[1], v1[2]);

            let (cx, sx) = (libm::cosf(angle_x), libm::sinf(angle_x));
            let (cy, sy) = (libm::cosf(angle_y), libm::sinf(angle_y));

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
            let px0 = map_to_dac(x0r * scale0);
            let py0 = map_to_dac(y0r * scale0);

            let scale1 = 2.0 / (2.0 + z1r2);
            let px1 = map_to_dac(x1r * scale1);
            let py1 = map_to_dac(y1r * scale1);

            self.points.push(Point {
                x: px0,
                y: py0,
                color: (0, 0, 0),
                delay: 400,
            });

            // Draw line with stepping
            let steps = 20;
            for s in 0..=steps {
                let t = s as f32 / steps as f32;

                // Interpolate
                let xi = px0 as f32 + t * (px1 as f32 - px0 as f32);
                let yi = py0 as f32 + t * (py1 as f32 - py0 as f32);

                self.points.push(Point {
                    x: xi as u8,
                    y: yi as u8,
                    color,
                    delay: 50,
                });
            }

            self.points.push(Point {
                x: px1,
                y: py1,
                color,
                delay: 100,
            });
        }

        self.points.extend(&self.static_points);

        &self.points
    }
}
