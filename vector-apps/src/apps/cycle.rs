use alloc::{boxed::Box, vec::Vec};

use crate::{apps::VectorApp, point::Path};

pub struct Cycle {
    apps: Vec<Box<dyn VectorApp>>,
}

impl Cycle {
    pub fn new(apps: Vec<Box<dyn VectorApp>>) -> Self {
        Self { apps }
    }
}

impl VectorApp for Cycle {
    fn get_path(&mut self, frame: u64) -> &Path {
        let idx = (frame / 50) as usize % self.apps.len();
        let app = &mut self.apps[idx];
        app.get_path(frame)
    }
}
