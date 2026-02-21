use alloc::vec::Vec;

use crate::{
    apps::{Controls, VectorApp},
    point::{Path, Point},
    utils::math::Vec2,
};

#[derive(Clone, Copy, Debug)]
struct LatLon {
    pub lon: f32,
    pub lat: f32,
}

struct Polyline<T> {
    points: Vec<T>,
    color: (u8, u8, u8),
}

type Polylines<T> = Vec<Polyline<T>>;

fn parse_latlon_file(data: &str) -> Polylines<LatLon> {
    let mut polylines = Vec::new();

    let mut current_points: Vec<LatLon> = Vec::new();
    let mut current_color: Option<(u8, u8, u8)> = None;

    for line in data.lines() {
        let line = line.trim();

        if line.is_empty() {
            if current_points.len() >= 2 {
                if let Some(color) = current_color {
                    polylines.push(Polyline {
                        points: current_points.clone(),
                        color,
                    });
                }
            }
            current_points.clear();
            continue;
        }

        // Color line: #RRGGBB
        if let Some(hex) = line.strip_prefix('#') {
            // Flush previous polyline if any
            if current_points.len() >= 2 {
                if let Some(color) = current_color {
                    polylines.push(Polyline {
                        points: current_points.clone(),
                        color,
                    });
                }
            }

            current_points.clear();

            let rgb = u32::from_str_radix(hex, 16).expect("invalid color");
            current_color = Some((
                ((rgb >> 16) & 0xff) as u8,
                ((rgb >> 8) & 0xff) as u8,
                (rgb & 0xff) as u8,
            ));
            continue;
        }

        // Coordinate line
        let mut it = line.split_whitespace();
        let lon: f32 = it.next().unwrap().parse().unwrap();
        let lat: f32 = it.next().unwrap().parse().unwrap();

        current_points.push(LatLon { lon, lat });
    }

    // Flush final block
    if current_points.len() >= 2 {
        if let Some(color) = current_color {
            polylines.push(Polyline {
                points: current_points,
                color,
            });
        }
    }

    polylines
}

const METERS_PER_DEG: f32 = 111_320.0;

fn project(ll: LatLon, lat0: f32, lon0: f32, cos_lat0: f32) -> Vec2 {
    Vec2 {
        x: (ll.lon - lon0) * cos_lat0 * METERS_PER_DEG,
        y: (ll.lat - lat0) * METERS_PER_DEG,
    }
}

fn clip_segment(a: Vec2, b: Vec2, half: f32) -> Option<(Vec2, Vec2)> {
    let dx = b.x - a.x;
    let dy = b.y - a.y;

    let mut t0 = 0.0;
    let mut t1 = 1.0;

    let checks = [
        (-dx, a.x + half),
        (dx, half - a.x),
        (-dy, a.y + half),
        (dy, half - a.y),
    ];

    for (p, q) in checks {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return None;
                }
                if r > t0 {
                    t0 = r;
                }
            } else {
                if r < t0 {
                    return None;
                }
                if r < t1 {
                    t1 = r;
                }
            }
        }
    }

    Some((
        Vec2 {
            x: a.x + t0 * dx,
            y: a.y + t0 * dy,
        },
        Vec2 {
            x: a.x + t1 * dx,
            y: a.y + t1 * dy,
        },
    ))
}

fn project_and_crop(
    input: &Polylines<LatLon>,
    lat0: f32,
    lon0: f32,
    side_m: f32,
) -> Polylines<Vec2> {
    let half = side_m * 0.5;
    let cos_lat0 = libm::cosf(lat0.to_radians());

    let mut out = Vec::new();

    for line in input {
        let mut current = Vec::new();

        for seg in line.points.windows(2) {
            let a = project(seg[0], lat0, lon0, cos_lat0);
            let b = project(seg[1], lat0, lon0, cos_lat0);

            if let Some((ca, cb)) = clip_segment(a, b, half) {
                if current.is_empty() {
                    current.push(ca);
                }
                current.push(cb);
            } else if current.len() >= 2 {
                out.push(Polyline {
                    points: current,
                    color: line.color,
                });
                current = Vec::new();
            }
        }

        if current.len() >= 2 {
            out.push(Polyline {
                points: current,
                color: line.color,
            });
        }
    }

    out
}

fn merge_connected_lines(mut lines: Polylines<Vec2>, tol: f32) -> Polylines<Vec2> {
    let tol2 = tol * tol;
    let mut merged = Vec::new();

    while let Some(mut line) = lines.pop() {
        let mut changed = true;

        while changed {
            changed = false;

            let mut i = 0;
            while i < lines.len() {
                let other = &lines[i];

                let (a0, a1) = (line.points[0], *line.points.last().unwrap());
                let (b0, b1) = (other.points[0], *other.points.last().unwrap());

                let matched = if line.color != other.color {
                    false
                } else if a1.distance(b0) < tol2 {
                    line.points.extend_from_slice(&other.points[1..]);
                    true
                } else if a1.distance(b1) < tol2 {
                    for p in other.points[..other.points.len() - 1].iter().rev() {
                        line.points.push(*p);
                    }
                    true
                } else if a0.distance(b1) < tol2 {
                    let mut new = other.points[..other.points.len() - 1].to_vec();
                    new.extend(line.points);
                    line = Polyline {
                        points: new,
                        color: line.color,
                    };
                    true
                } else if a0.distance(b0) < tol2 {
                    let mut new = other.points[1..].iter().rev().cloned().collect::<Vec<_>>();
                    new.extend(line.points);
                    line = Polyline {
                        points: new,
                        color: line.color,
                    };
                    true
                } else {
                    false
                };

                if matched {
                    lines.swap_remove(i);
                    changed = true;
                    break;
                } else {
                    i += 1;
                }
            }
        }

        merged.push(line);
    }

    merged
}

fn greedy_order_lines(mut lines: Polylines<Vec2>) -> Polylines<Vec2> {
    if lines.is_empty() {
        return lines;
    }

    let mut ordered = Vec::new();
    ordered.push(lines.pop().unwrap());

    while !lines.is_empty() {
        let last = *ordered.last().unwrap().points.last().unwrap();

        let mut best_i = 0;
        let mut best_d = f32::MAX;
        let mut reverse = false;

        for (i, line) in lines.iter().enumerate() {
            let d0 = last.distance(line.points[0]);
            let d1 = last.distance(*line.points.last().unwrap());

            if d0 < best_d {
                best_d = d0;
                best_i = i;
                reverse = false;
            }
            if d1 < best_d {
                best_d = d1;
                best_i = i;
                reverse = true;
            }
        }

        let mut next = lines.swap_remove(best_i);
        if reverse {
            next.points.reverse();
        }
        ordered.push(next);
    }

    ordered
}

fn normalize(lines: &mut Polylines<Vec2>, side_m: f32) {
    let half = side_m * 0.5;
    for line in lines {
        for p in &mut line.points {
            p.x /= half;
            p.y /= half;
        }
    }
}

pub struct Mbta {
    path: Path,
}

const SIDE_METERS: f32 = 20000.0;

const BLANK_COLOR: (u8, u8, u8) = (0, 0, 0);

const CENTER_LAT: f32 = 42.35872301447337;
const CENTER_LON: f32 = -71.05748969650732;

impl Mbta {
    pub fn new() -> Self {
        let path: Path = Vec::new();

        let mut map = Self { path };
        map.generate_path();
        map
    }

    fn generate_path(&mut self) {
        self.path.clear();

        let raw = include_str!("mbta.txt");
        let latlon = parse_latlon_file(raw);

        let mut lines = project_and_crop(&latlon, CENTER_LAT, CENTER_LON, SIDE_METERS);
        lines = merge_connected_lines(lines, 0.5); // 0.5m tolerance
        lines = greedy_order_lines(lines);
        normalize(&mut lines, SIDE_METERS);

        let mut prev: Option<(u8, u8)> = None;

        // Max step distance for interpolation (DAC units)
        const STEP: f32 = 1.0;

        for line in &lines {
            if line.points.len() < 2 {
                continue;
            }

            for (i, p) in line.points.iter().enumerate() {
                // Map normalized [-1,1] → DAC [0,255]
                let x = libm::roundf((p.x + 1.0) * 0.5 * 255.0) as u8;
                let y = libm::roundf((-p.y + 1.0) * 0.5 * 255.0) as u8;

                if i == 0 {
                    // Move laser (blanked) to start of polyline
                    let delay = match prev {
                        Some((px, py)) => {
                            let dx = x as f32 - px as f32;
                            let dy = y as f32 - py as f32;
                            libm::sqrtf(dx * dx + dy * dy) as u16
                        }
                        None => 1000,
                    } + 500;

                    // Pen-up move
                    self.path.push(Point {
                        x,
                        y,
                        color: BLANK_COLOR,
                        delay,
                    });

                    // Turn laser on
                    self.path.push(Point {
                        x,
                        y,
                        color: line.color,
                        delay: 100,
                    });

                    prev = Some((x, y));
                } else {
                    // Interpolate from previous point
                    let (px, py) = prev.unwrap();
                    let dx = x as f32 - px as f32;
                    let dy = y as f32 - py as f32;
                    let distance = libm::sqrtf(dx * dx + dy * dy);

                    let steps = libm::ceilf(distance / STEP) as u16;

                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        let ix = libm::roundf(px as f32 + dx * t) as u8;
                        let iy = libm::roundf(py as f32 + dy * t) as u8;

                        self.path.push(Point {
                            x: ix,
                            y: iy,
                            color: line.color,
                            delay: 1,
                        });
                    }

                    // Small settle at the vertex
                    self.path.push(Point {
                        x,
                        y,
                        color: line.color,
                        delay: 100,
                    });

                    prev = Some((x, y));
                }
            }
        }

        self.path.push(Point {
            x: 128,
            y: 128,
            color: BLANK_COLOR,
            delay: 1,
        });
    }
}

impl VectorApp for Mbta {
    fn get_path(&mut self, _frame: u64) -> &Path {
        &self.path
    }

    fn handle_controls(&mut self, controls: Controls) {
        // let cos_lat = libm::cosf(self.lat.to_radians());

        // self.lat += controls.y as f32 * 1000.0 / METERS_PER_DEG;
        // self.lon += controls.x as f32 * 1000.0 / (METERS_PER_DEG * cos_lat);

        // self.generate_path();
    }
}
