#![no_std]

extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Copy, Clone)]
struct PackedPoint {
    pub x: i8,
    pub y: i8,
    pub pen: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: i16,
    pub y: i16,
    pub pen: bool,
}

impl Default for Point {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            pen: false,
        }
    }
}

#[derive(Debug, Clone)]
struct Glyph {
    pub left: i8,
    pub right: i8,
    pub strokes: Vec<PackedPoint>,
}

impl Glyph {
    fn from_line(line: &str) -> Result<(u16, Self), ()> {
        let mut chars = line.chars();

        let id: String = chars.by_ref().take(5).collect();
        let id: u16 = id.trim().parse().map_err(|_| ())?;
        let _space = chars.next();
        let _vertex_count: String = chars.by_ref().take(2).collect();

        let coords: Vec<char> = chars.collect();

        let left = coords[0] as i32 - 'R' as i32;
        let right = coords[1] as i32 - 'R' as i32;

        let mut strokes = Vec::new();
        let mut pen = false;

        let mut iter = coords[2..].iter();

        while let (Some(&xch), Some(&ych)) = (iter.next(), iter.next()) {
            if xch == ' ' && ych == 'R' {
                // lift pen for the next stroke
                pen = false;
                continue;
            }

            let x = xch as i32 - 'R' as i32;
            let y = ych as i32 - 'R' as i32;
            strokes.push(PackedPoint {
                x: x as i8,
                y: y as i8,
                pen,
            });
            // drop the pen for the rest of this stroke
            pen = true;
        }

        Ok((
            id,
            Self {
                left: left as i8,
                right: right as i8,
                strokes,
            },
        ))
    }
}

const NUM_GLYPHS: usize = 2500;
type FontFile = [Option<Box<Glyph>>; NUM_GLYPHS];

fn load_file(file: &str) -> FontFile {
    let mut result = [const { None }; NUM_GLYPHS];
    let mut lines = file.lines();

    loop {
        let next_line = lines.next();

        match next_line {
            Some("") => continue,
            Some(line) => {
                let mut full = line.to_owned();
                let mut last_line = line;

                while last_line.len() == 72 {
                    let line = lines.next().unwrap_or("");
                    full += line;
                    last_line = line;
                }

                let glyph = Glyph::from_line(&full);

                if let Ok((id, glyph)) = glyph {
                    let id = id as usize;
                    if id < NUM_GLYPHS {
                        result[id] = Some(Box::new(glyph));
                    }
                }
            }
            None => break result,
        }
    }
}

pub type FontMapping = [u16; 256];

pub fn load_mapping(file: &str) -> Result<FontMapping, ()> {
    let mut result = [0; 256];
    let mut codepoint: usize = 32;

    for line in file.lines() {
        if line.is_empty() {
            continue;
        }

        let mut parts = line.split(" ");

        if let Some(first) = parts.next() {
            if let Some(last) = parts.next() {
                if let Ok(first) = first.parse::<usize>() {
                    if let Ok(mut last) = last.parse::<usize>() {
                        if last == 0 {
                            last = first;
                        }

                        for idx in first..=last {
                            result[codepoint] = idx as u16;
                            codepoint += 1;
                        }
                    }
                }
            }
        }

        // let first: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
        // let mut last: usize = parts.next().ok_or(())?.parse().map_err(|_| ())?;
    }

    Ok(result)
}

pub struct HersheyRenderer {
    font: FontFile,
}

impl Default for HersheyRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl HersheyRenderer {
    pub fn new() -> Self {
        Self {
            font: load_file(HERSHEY_SOURCE),
        }
    }

    pub fn render_text(&self, text: &str, mapping: FontMapping) -> Vec<Point> {
        let mut result = Vec::new();
        let mut x_idx = 0;

        for character in text.chars() {
            if character > 255 as char {
                continue;
            }

            let hershey_id = mapping[character as usize] as usize;

            if hershey_id == 0 || hershey_id >= NUM_GLYPHS {
                continue;
            }

            if let Some(glyph) = &self.font[hershey_id] {
                result.extend(glyph.strokes.iter().map(|point| Point {
                    x: point.x as i16 - glyph.left as i16 + x_idx,
                    y: point.y as i16,
                    pen: point.pen,
                }));
                x_idx += glyph.right as i16 - glyph.left as i16;
            }
        }

        result
    }
}

const HERSHEY_SOURCE: &str = include_str!("data/hershey.jhf");

pub mod fonts {
    pub const ASTROL: &str = include_str!("mappings/astrol.hmp");
    pub const CYRILC: &str = include_str!("mappings/cyrilc.hmp");

    pub const GOTHENG: &str = include_str!("mappings/gotheng.hmp");
    pub const GOTHGER: &str = include_str!("mappings/gothger.hmp");
    pub const GOTHITA: &str = include_str!("mappings/gothita.hmp");

    pub const GREEKCS: &str = include_str!("mappings/greekcs.hmp");
    pub const GREEKC: &str = include_str!("mappings/greekc.hmp");
    pub const GREEKP: &str = include_str!("mappings/greekp.hmp");
    pub const GREEKS: &str = include_str!("mappings/greeks.hmp");

    pub const ITALICCS: &str = include_str!("mappings/italiccs.hmp");
    pub const ITALICC: &str = include_str!("mappings/italicc.hmp");
    pub const ITALICT: &str = include_str!("mappings/italict.hmp");

    pub const JAPAN: &str = include_str!("mappings/japan.hmp");

    pub const LOWMAT: &str = include_str!("mappings/lowmat.hmp");
    pub const UPPMAT: &str = include_str!("mappings/uppmat.hmp");

    pub const MARKER: &str = include_str!("mappings/marker.hmp");
    pub const METEO: &str = include_str!("mappings/meteo.hmp");
    pub const MISC: &str = include_str!("mappings/misc.hmp");
    pub const MUSIC: &str = include_str!("mappings/music.hmp");

    pub const ROMANC: &str = include_str!("mappings/romanc.hmp");
    pub const ROMANCS: &str = include_str!("mappings/romancs.hmp");
    pub const ROMAND: &str = include_str!("mappings/romand.hmp");
    pub const ROMANP: &str = include_str!("mappings/romanp.hmp");
    pub const ROMANS: &str = include_str!("mappings/romans.hmp");
    pub const ROMANT: &str = include_str!("mappings/romant.hmp");

    pub const SCRIPTC: &str = include_str!("mappings/scriptc.hmp");
    pub const SCRIPTS: &str = include_str!("mappings/scripts.hmp");

    pub const SYMBOL: &str = include_str!("mappings/symbol.hmp");
}
