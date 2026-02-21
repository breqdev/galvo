use alloc::vec::Vec;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
};

pub struct Align {
    points: Vec<Point>,
}

impl Align {
    pub fn new() -> Self {
        let mut points = Vec::new();

        points.push(Point {
            x: 128,
            y: 128,
            color: (64, 128, 64),
            delay: 1000,
        });

        Self { points }
    }
}

impl VectorApp for Align {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.points
    }
}
