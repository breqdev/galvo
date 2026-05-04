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
pub mod svg;

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

impl Controls {
    pub fn merge(self, other: Controls) -> Controls {
        Controls {
            x: (self.x as i16 + other.x as i16).clamp(-127, 127) as i8,
            y: (self.y as i16 + other.y as i16).clamp(-127, 127) as i8,
            a: self.a || other.a,
            b: self.b || other.b,
        }
    }
}

pub trait VectorApp {
    fn get_path(&mut self, frame: u64) -> &Path;

    fn handle_controls(&mut self, _controls: Controls) {}
}
