use crate::point::Path;

pub mod alphabet;
pub mod asteroids;
pub mod cube;

pub trait VectorApp {
    fn get_path(&mut self, frame: u64) -> &Path;
}
