use alloc::vec::Vec;

use crate::point::Point;

pub mod alphabet;
pub mod cube;

pub trait Demo {
    fn get_path(&self, frame: u64) -> Vec<Point>;
}
