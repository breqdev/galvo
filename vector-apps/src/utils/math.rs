#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    pub fn mul(self, s: f32) -> Vec2 {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }

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

fn wrap(v: f32) -> f32 {
    if v < 0.0 {
        v + 1.0
    } else if v > 1.0 {
        v - 1.0
    } else {
        v
    }
}
