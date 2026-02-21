use crate::point::Path;

pub mod align;
pub mod alphabet;
pub mod asteroids;
pub mod clock;
pub mod cube;
pub mod cycle;
pub mod ilda;
pub mod maps;
pub mod mbta;

#[derive(Clone, Copy, Debug, Default)]
pub struct Controls {
    /// The X-axis (horizontal) control.
    pub x: i8,

    /// The Y-axis (vertical) control.
    pub y: i8,

    /// The primary button input.
    pub a: bool,

    /// The secondary button input.
    pub b: bool,
}

pub trait VectorApp {
    fn get_path(&mut self, frame: u64) -> &Path;

    fn handle_controls(&mut self, _controls: Controls) {}
}
