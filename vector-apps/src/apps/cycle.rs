use alloc::{boxed::Box, vec::Vec};

use crate::{
    apps::{Controls, VectorApp},
    point::Path,
};

pub struct Cycle {
    apps: Vec<Box<dyn VectorApp>>,
    idx: usize,
    b_pressed: bool,
}

impl Cycle {
    pub fn new(apps: Vec<Box<dyn VectorApp>>) -> Self {
        Self {
            apps,
            idx: 0,
            b_pressed: false,
        }
    }
}

impl VectorApp for Cycle {
    fn get_path(&mut self, frame: u64) -> &Path {
        let app = &mut self.apps[self.idx];
        app.get_path(frame)
    }

    fn handle_controls(&mut self, controls: Controls) {
        if controls.b && !self.b_pressed {
            self.idx += 1;
            self.idx %= self.apps.len();
        } else {
            self.apps[self.idx].handle_controls(controls);
        }

        self.b_pressed = controls.b;
    }
}
