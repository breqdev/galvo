use alloc::vec::Vec;
use hershey_text::fonts;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
    utils::text::text_to_path,
};

pub struct AlphabetDemo {
    points: Vec<Point>,
}

impl AlphabetDemo {
    pub fn new() -> Self {
        let color = 1;

        let mut points = text_to_path("12345678", 0, 32, 1.0, 1.0, color, fonts::ROMANS);

        points.append(&mut text_to_path(
            "12345678",
            0,
            64,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));
        points.append(&mut text_to_path(
            "!@#$%^&*",
            0,
            96,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));
        points.append(&mut text_to_path(
            "ABCDEFGH",
            0,
            128,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));
        points.append(&mut text_to_path(
            "ABCDEFGH",
            0,
            160,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));
        // points.append(&mut text_to_path("abcdefgh", 255, 128, -1.0, 1.0));
        points.append(&mut text_to_path(
            "abcdefgh",
            0,
            192,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));
        points.append(&mut text_to_path(
            "abcdefgh",
            0,
            224,
            1.0,
            1.0,
            color,
            fonts::ROMANS,
        ));

        Self { points }
    }
}

impl VectorApp for AlphabetDemo {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.points
    }
}
