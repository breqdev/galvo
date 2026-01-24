use core::ops::{Add, Mul};

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn wrap(self) -> Vec2 {
        Vec2 {
            x: wrap(self.x),
            y: wrap(self.y),
        }
    }

    pub fn distance(self, other: Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, s: f32) -> Vec2 {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

fn wrap(v: f32) -> f32 {
    if v < 0.0 {
        v + 1.0
    } else if v > 1.0 {
        v - 1.0
    } else {
        v
    }
}
