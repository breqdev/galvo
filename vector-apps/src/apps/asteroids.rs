use core::f32::consts::TAU;

use alloc::vec::Vec;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
};

#[derive(Clone, Copy)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    fn add(self, o: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + o.x,
            y: self.y + o.y,
        }
    }

    fn mul(self, s: f32) -> Vec2 {
        Vec2 {
            x: self.x * s,
            y: self.y * s,
        }
    }
}

fn dist2(a: Vec2, b: Vec2) -> f32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
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

struct Ship {
    pos: Vec2,
    rot: f32,
}

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    ttl: u16,
}

#[derive(Clone, Copy)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(self) -> f32 {
        match self {
            AsteroidSize::Large => 0.06,
            AsteroidSize::Medium => 0.04,
            AsteroidSize::Small => 0.025,
        }
    }

    fn fragments(self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }
}

struct Asteroid {
    pos: Vec2,
    vel: Vec2,
    size: AsteroidSize,
}

pub struct Asteroids {
    ship: Ship,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,
    path: Path,
}

impl Asteroids {
    pub fn new() -> Self {
        let mut asteroids = Vec::new();

        asteroids.push(Asteroid {
            pos: Vec2 { x: 0.2, y: 0.3 },
            vel: Vec2 {
                x: 0.0007,
                y: 0.0003,
            },
            // vel: Vec2 { x: 0.0, y: 0.0 },
            size: AsteroidSize::Large,
        });

        asteroids.push(Asteroid {
            pos: Vec2 { x: 0.8, y: 0.6 },
            vel: Vec2 {
                x: -0.0004,
                y: 0.0006,
            },
            // vel: Vec2 { x: 0.0, y: 0.0 },
            size: AsteroidSize::Medium,
        });

        Self {
            ship: Ship {
                pos: Vec2 { x: 0.5, y: 0.5 },
                rot: 0.0,
            },
            asteroids,
            bullets: Vec::new(),
            path: Vec::new(),
        }
    }

    fn step(&mut self) {
        // rotate ship slowly
        self.ship.rot += 0.002;

        // drift asteroids
        for a in &mut self.asteroids {
            a.pos = a.pos.add(a.vel);
            a.pos.x = wrap(a.pos.x);
            a.pos.y = wrap(a.pos.y);
        }

        // TODO: actually shoot bullets when controlled
        // rn its just arbitrary
        if self.bullets.len() < 1 {
            let forward = Vec2 {
                x: libm::sinf(self.ship.rot),
                y: libm::cosf(self.ship.rot),
            };

            self.bullets.push(Bullet {
                pos: self.ship.pos,
                vel: forward.mul(0.01),
                ttl: 400,
            });
        }

        // move bullets
        for b in &mut self.bullets {
            b.pos = b.pos.add(b.vel);
            b.pos.x = wrap(b.pos.x);
            b.pos.y = wrap(b.pos.y);
            b.ttl -= 1;
        }

        // remove expired bullets
        self.bullets.retain(|b| b.ttl > 0);

        // collision!
        let mut new_asteroids = Vec::new();

        self.asteroids.retain(|a| {
            let mut hit = false;

            self.bullets.retain(|b| {
                let r = a.size.radius();
                if dist2(a.pos, b.pos) < r * r {
                    hit = true;
                    false // remove bullet
                } else {
                    true
                }
            });

            if hit {
                if let Some(next) = a.size.fragments() {
                    for i in 0..2 {
                        let angle = (i as f32) * 1.7;
                        new_asteroids.push(Asteroid {
                            pos: a.pos,
                            vel: Vec2 {
                                x: libm::cosf(angle) * 0.001,
                                y: libm::sinf(angle) * 0.001,
                            },
                            size: next,
                        });
                    }
                }
                false // remove asteroid
            } else {
                true
            }
        });

        self.asteroids.extend(new_asteroids);
    }

    fn draw_circle(&mut self, center: Vec2, r: f32, color: u8) {
        const SEGMENTS: usize = 24;
        let mut prev = None;
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;

        for i in 0..=SEGMENTS {
            let a = i as f32 / SEGMENTS as f32 * TAU;
            let p = Vec2 {
                x: center.x + r * libm::cosf(a),
                y: center.y + r * libm::sinf(a),
            };

            if let Some(_last) = prev {
                self.path.push(Point {
                    x: to_u8(p.x),
                    y: to_u8(p.y),
                    color,
                    delay: 100,
                });
            } else {
                self.path.push(Point {
                    x: to_u8(p.x),
                    y: to_u8(p.y),
                    color: 0,
                    delay: 1000,
                });
            }
            prev = Some(p);
        }
    }

    fn draw_ship(&mut self) {
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;

        let forward = Vec2 {
            x: libm::sinf(self.ship.rot),
            y: libm::cosf(self.ship.rot),
        };
        let left = Vec2 {
            x: libm::sinf(self.ship.rot + 2.5),
            y: libm::cosf(self.ship.rot + 2.5),
        };
        let right = Vec2 {
            x: libm::sinf(self.ship.rot - 2.5),
            y: libm::cosf(self.ship.rot - 2.5),
        };

        let p0 = self.ship.pos.add(forward.mul(0.03));
        let p1 = self.ship.pos.add(left.mul(0.02));
        let p2 = self.ship.pos.add(right.mul(0.02));

        self.path.push(Point {
            x: to_u8(p0.x),
            y: to_u8(p0.y),
            color: 0,
            delay: 1000,
        });
        self.path.push(Point {
            x: to_u8(p1.x),
            y: to_u8(p1.y),
            color: 255,
            delay: 500,
        });
        self.path.push(Point {
            x: to_u8(p2.x),
            y: to_u8(p2.y),
            color: 255,
            delay: 500,
        });
        self.path.push(Point {
            x: to_u8(p0.x),
            y: to_u8(p0.y),
            color: 255,
            delay: 500,
        });
    }

    fn render(&mut self) {
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;

        self.path.clear();

        self.draw_ship();

        let len = self.asteroids.len();
        for i in 0..len {
            let (pos, radius) = {
                let a = &self.asteroids[i];
                (a.pos, a.size.radius())
            };
            self.draw_circle(pos, radius, 1);
        }

        for b in &self.bullets {
            self.path.push(Point {
                x: to_u8(b.pos.x),
                y: to_u8(b.pos.y),
                color: 0,
                delay: 1000,
            });
            self.path.push(Point {
                x: to_u8(b.pos.x),
                y: to_u8(b.pos.y),
                color: 1,
                delay: 100,
            });
            self.path.push(Point {
                x: to_u8(b.pos.x + b.vel.x),
                y: to_u8(b.pos.y + b.vel.y),
                color: 1,
                delay: 200,
            });
        }

        // move the beam back to the center
        // so that time between frames doesn't cause a bright spot
        self.path.push(Point {
            x: 128,
            y: 128,
            color: 0,
            delay: 0,
        })
    }
}

impl VectorApp for Asteroids {
    fn get_path(&mut self, _frame: u64) -> &Path {
        self.step();
        self.render();
        &self.path
    }
}
