use alloc::vec::Vec;
use hershey_text::fonts;

use crate::{demos::Demo, point::Point, text::text_to_path};

pub struct AlphabetDemo {
    points: Vec<Point>,
}

impl AlphabetDemo {
    pub fn new() -> Self {
        let mut points = text_to_path("12345678", 0, 32, 1.0, 1.0, fonts::ROMANS);

        points.append(&mut text_to_path(
            "12345678",
            0,
            64,
            1.0,
            1.0,
            fonts::ROMANS,
        ));
        // points.append(&mut text_to_path(
        //     "!@#$%^&*",
        //     0,
        //     96,
        //     1.0,
        //     1.0,
        //     fonts::ROMANS,
        // ));
        points.append(&mut text_to_path(
            "ABCDEFGH",
            0,
            128,
            1.0,
            1.0,
            fonts::ROMANS,
        ));
        points.append(&mut text_to_path(
            "ABCDEFGH",
            0,
            160,
            1.0,
            1.0,
            fonts::ROMANS,
        ));
        // points.append(&mut text_to_path("abcdefgh", 255, 128, -1.0, 1.0));
        points.append(&mut text_to_path(
            "abcdefgh",
            0,
            192,
            1.0,
            1.0,
            fonts::ROMANS,
        ));
        // points.append(&mut text_to_path(
        //     "abcdefgh",
        //     0,
        //     224,
        //     1.0,
        //     1.0,
        //     fonts::ROMANS,
        // ));

        Self { points }
    }
}

impl Demo for AlphabetDemo {
    fn get_path(&self, _frame: u64) -> Vec<Point> {
        self.points.clone()
    }
}
