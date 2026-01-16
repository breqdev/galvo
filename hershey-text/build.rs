use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Copy, Clone)]
struct PackedPoint {
    pub x: i8,
    pub y: i8,
    pub pen: bool,
}
const NUM_GLYPHS: usize = 2500;
type FontFile = [Option<Glyph>; NUM_GLYPHS];

fn generate_rust(font: &[Option<Glyph>], mappings: &HashMap<String, FontMapping>) -> String {
    let mut out = String::new();

    // Write the symbol table
    out.push_str(&format!(
        "static HERSHEY_FONT: [Option<Glyph>; {}] = [\n",
        NUM_GLYPHS
    ));

    for glyph in font {
        match glyph {
            None => out.push_str("    None,\n"),
            Some(g) => {
                out.push_str("    Some(Glyph {\n");
                out.push_str(&format!("        left: {},\n", g.left));
                out.push_str(&format!("        right: {},\n", g.right));
                out.push_str("        strokes: &[\n");

                for p in &g.strokes {
                    out.push_str(&format!(
                        "            PackedPoint {{ x: {}, y: {}, pen: {} }},\n",
                        p.x, p.y, p.pen
                    ));
                }

                out.push_str("        ],\n    }),\n");
            }
        }
    }

    out.push_str("];\n");

    // Write the font lookup tables
    out.push_str("pub mod fonts {\n");

    for (name, data) in mappings {
        let parts: Vec<_> = name.split(".").collect();

        out.push_str(&format!(
            "  pub static {}: [u16; 256] = [\n    ",
            parts[0].to_uppercase()
        ));

        for (i, v) in data.iter().enumerate() {
            out.push_str(&format!("{},\t", v));
            if i % 16 == 15 {
                out.push_str("\n    ");
            }
        }

        out.push_str("\n  ];\n\n");
    }

    out.push_str("}\n");

    out
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
                        result[id] = Some(glyph);
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
                            if codepoint < 256 {
                                result[codepoint] = idx as u16;
                            }
                            codepoint += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(result)
}

fn main() {
    let hershey = fs::read_to_string("data/hershey.jhf").unwrap();

    let glyphs = load_file(&hershey);

    let mut mappings: HashMap<String, FontMapping> = HashMap::new();

    for file in fs::read_dir("data/mappings").unwrap() {
        let file = file.unwrap();
        let contents = fs::read_to_string(file.path()).unwrap();
        let result = load_mapping(&contents);
        mappings.insert(file.file_name().into_string().unwrap(), result.unwrap());
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let out_file = out_dir.join("hershey_font.rs");

    fs::write(out_file, generate_rust(&glyphs, &mappings)).unwrap();

    println!("cargo:rerun-if-changed=data/hershey.jhf");
}
