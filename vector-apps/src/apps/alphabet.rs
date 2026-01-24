use alloc::{string::String, vec::Vec};
use hershey_text::fonts;
use itertools::Itertools;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
    utils::text::text_to_path,
};

pub struct AlphabetDemo {
    points: Vec<Point>,
}

impl AlphabetDemo {
    pub fn new(text: String) -> Self {
        let color = (255, 0, 0);

        let points = text
            .chars()
            .chunks(8)
            .into_iter()
            .take(5)
            .enumerate()
            .map(|(i, mut line)| {
                text_to_path(
                    &line.join(""),
                    0,
                    32 + 32 * i as u8,
                    1.0,
                    1.0,
                    color,
                    fonts::ROMANS,
                )
            })
            .flatten()
            .collect();

        Self { points }
    }
}

impl VectorApp for AlphabetDemo {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.points
    }
}
