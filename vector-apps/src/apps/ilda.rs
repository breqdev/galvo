use alloc::vec::Vec;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
    utils::ilda::read_ilda,
};

pub struct Ilda {
    points: Vec<Point>,
}

const ILDA_KPPS: u8 = 12;

impl Ilda {
    pub fn new() -> Self {
        let paths = read_ilda(include_bytes!("ildatest.ild"), ILDA_KPPS);
        let points = paths.get("ILDA Tes").unwrap().to_vec();

        Self { points }
    }
}

impl VectorApp for Ilda {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.points
    }
}
