use core::str;

use alloc::{
    collections::btree_map::BTreeMap,
    string::{String, ToString},
    vec::Vec,
};

use crate::point::{Path, Point};

struct Cursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn read(&mut self, out: &mut [u8]) {
        let end = self.pos + out.len();
        if end > self.buf.len() {
            panic!();
        }
        out.copy_from_slice(&self.buf[self.pos..end]);
        self.pos = end;
    }

    fn read_u8(&mut self) -> u8 {
        if self.pos >= self.buf.len() {
            panic!();
        }
        let v = self.buf[self.pos];
        self.pos += 1;
        v
    }

    fn read_u16_be(&mut self) -> u16 {
        let end = self.pos + 2;
        if end > self.buf.len() {
            panic!();
        }
        let v = u16::from_be_bytes(self.buf[self.pos..end].try_into().unwrap());
        self.pos = end;
        v
    }

    fn read_i16_be(&mut self) -> i16 {
        let end = self.pos + 2;
        if end > self.buf.len() {
            panic!();
        }
        let v = i16::from_be_bytes(self.buf[self.pos..end].try_into().unwrap());
        self.pos = end;
        v
    }

    fn read_string(&mut self, chars: usize) -> String {
        let end = self.pos + chars;
        if end > self.buf.len() {
            panic!();
        }
        let v = str::from_utf8(&self.buf[self.pos..end])
            .unwrap()
            .to_string();
        self.pos = end;
        v
    }

    fn skip(&mut self, n: usize) {
        let end = self.pos + n;
        if end > self.buf.len() {
            panic!();
        }
        self.pos = end;
    }
}

struct IldaHeader {
    format: u8,
    name: String,
    author: String,
    num_records: u16,
    frame_number: u16,
    total_frames: u16,
    projector_number: u8,
}

const LASTPOINT_BIT: u8 = 0b10000000;
const BLANKING_BIT: u8 = 0b01000000;

pub const ILDA_DEFAULT_PALETTE: [(u8, u8, u8); 64] = [
    (255, 0, 0),     //  0 Red
    (255, 16, 0),    //  1
    (255, 32, 0),    //  2
    (255, 48, 0),    //  3
    (255, 64, 0),    //  4
    (255, 80, 0),    //  5
    (255, 96, 0),    //  6
    (255, 112, 0),   //  7
    (255, 128, 0),   //  8
    (255, 144, 0),   //  9
    (255, 160, 0),   // 10
    (255, 176, 0),   // 11
    (255, 192, 0),   // 12
    (255, 208, 0),   // 13
    (255, 224, 0),   // 14
    (255, 240, 0),   // 15
    (255, 255, 0),   // 16 Yellow
    (224, 255, 0),   // 17
    (192, 255, 0),   // 18
    (160, 255, 0),   // 19
    (128, 255, 0),   // 20
    (96, 255, 0),    // 21
    (64, 255, 0),    // 22
    (32, 255, 0),    // 23
    (0, 255, 0),     // 24 Green
    (0, 255, 36),    // 25
    (0, 255, 73),    // 26
    (0, 255, 109),   // 27
    (0, 255, 146),   // 28
    (0, 255, 182),   // 29
    (0, 255, 219),   // 30
    (0, 255, 255),   // 31 Cyan
    (0, 227, 255),   // 32
    (0, 198, 255),   // 33
    (0, 170, 255),   // 34
    (0, 142, 255),   // 35
    (0, 113, 255),   // 36
    (0, 85, 255),    // 37
    (0, 56, 255),    // 38
    (0, 28, 255),    // 39
    (0, 0, 255),     // 40 Blue
    (32, 0, 255),    // 41
    (64, 0, 255),    // 42
    (96, 0, 255),    // 43
    (128, 0, 255),   // 44
    (160, 0, 255),   // 45
    (192, 0, 255),   // 46
    (224, 0, 255),   // 47
    (255, 0, 255),   // 48 Magenta
    (255, 32, 255),  // 49
    (255, 64, 255),  // 50
    (255, 96, 255),  // 51
    (255, 128, 255), // 52
    (255, 160, 255), // 53
    (255, 192, 255), // 54
    (255, 224, 255), // 55
    (255, 255, 255), // 56 White
    (255, 224, 224), // 57
    (255, 192, 192), // 58
    (255, 160, 160), // 59
    (255, 128, 128), // 60
    (255, 96, 96),   // 61
    (255, 64, 64),   // 62
    (255, 32, 32),   // 63
];

struct Parser<'a> {
    cur: Cursor<'a>,
    palette: Vec<(u8, u8, u8)>,
    delay: u16,
}

impl<'a> Parser<'a> {
    fn new(data: &'a [u8], kpps: u8) -> Self {
        let cur = Cursor::new(data);

        let delay = (1000.0 / kpps as f32) as u16;

        Parser {
            cur,
            palette: ILDA_DEFAULT_PALETTE.to_vec(),
            delay,
        }
    }

    fn parse_header(&mut self) -> IldaHeader {
        let mut magic = [0; 4];
        self.cur.read(&mut magic);
        if magic != *"ILDA".as_bytes() {
            panic!();
        }

        self.cur.skip(3);

        let result = IldaHeader {
            format: self.cur.read_u8(),
            name: self.cur.read_string(8),
            author: self.cur.read_string(8),
            num_records: self.cur.read_u16_be(),
            frame_number: self.cur.read_u16_be(),
            total_frames: self.cur.read_u16_be(),
            projector_number: self.cur.read_u8(),
        };

        self.cur.skip(1);
        result
    }

    fn parse_record_fmt0(&mut self) -> Point {
        let x = self.cur.read_i16_be();
        let y = self.cur.read_i16_be();
        let _z = self.cur.read_i16_be();
        let status = self.cur.read_u8();
        let color_idx = self.cur.read_u8() as usize;

        Point {
            x: ((x as i32 + 32768) >> 8) as u8,
            y: 255 - ((y as i32 + 32768) >> 8) as u8,
            color: if status & BLANKING_BIT == 0 {
                *self.palette.get(color_idx).unwrap_or(&(0, 0, 0))
            } else {
                (0, 0, 0)
            },
            delay: self.delay,
        }
    }

    fn parse_record_fmt1(&mut self) -> Point {
        let x = self.cur.read_i16_be();
        let y = self.cur.read_i16_be();
        let status = self.cur.read_u8();
        let color_idx = self.cur.read_u8() as usize;

        Point {
            x: ((x as i32 + 32768) >> 8) as u8,
            y: 255 - ((y as i32 + 32768) >> 8) as u8,
            color: if status & BLANKING_BIT == 0 {
                *self.palette.get(color_idx).unwrap_or(&(0, 0, 0))
            } else {
                (0, 0, 0)
            },
            delay: self.delay,
        }
    }

    fn parse_record_fmt2(&mut self) -> (u8, u8, u8) {
        (self.cur.read_u8(), self.cur.read_u8(), self.cur.read_u8())
    }

    fn parse_record_fmt4(&mut self) -> Point {
        let x = self.cur.read_i16_be();
        let y = self.cur.read_i16_be();
        let _z = self.cur.read_i16_be();
        let status = self.cur.read_u8();
        let red = self.cur.read_u8();
        let green = self.cur.read_u8();
        let blue = self.cur.read_u8();

        Point {
            x: ((x as i32 + 32768) >> 8) as u8,
            y: 255 - ((y as i32 + 32768) >> 8) as u8,
            color: if status & BLANKING_BIT == 0 {
                (red, green, blue)
            } else {
                (0, 0, 0)
            },
            delay: self.delay,
        }
    }

    fn parse_record_fmt5(&mut self) -> Point {
        let x = self.cur.read_i16_be();
        let y = self.cur.read_i16_be();
        let status = self.cur.read_u8();
        let red = self.cur.read_u8();
        let green = self.cur.read_u8();
        let blue = self.cur.read_u8();

        Point {
            x: ((x as i32 + 32768) >> 8) as u8,
            y: 255 - ((y as i32 + 32768) >> 8) as u8,
            color: if status & BLANKING_BIT == 0 {
                (red, green, blue)
            } else {
                (0, 0, 0)
            },
            delay: self.delay,
        }
    }

    fn parse_file(&mut self) -> BTreeMap<String, Path> {
        let mut paths = BTreeMap::new();

        loop {
            let header = self.parse_header();

            if header.num_records == 0 {
                return paths;
            }

            match header.format {
                0 => {
                    let path: Path = (0..header.num_records)
                        .map(|_| self.parse_record_fmt0())
                        .collect();

                    paths.insert(header.name, path);
                }
                1 => {
                    let path: Path = (0..header.num_records)
                        .map(|_| self.parse_record_fmt1())
                        .collect();

                    paths.insert(header.name, path);
                }
                2 => {
                    self.palette = (0..header.num_records)
                        .map(|_| self.parse_record_fmt2())
                        .collect();
                }
                3 => panic!("invalid format code"),
                4 => {
                    let path: Path = (0..header.num_records)
                        .map(|_| self.parse_record_fmt4())
                        .collect();

                    paths.insert(header.name, path);
                }
                5 => {
                    let path: Path = (0..header.num_records)
                        .map(|_| self.parse_record_fmt5())
                        .collect();

                    paths.insert(header.name, path);
                }
                _ => panic!("invalid format code"),
            }
        }
    }
}

pub fn read_ilda(source: &[u8], kpps: u8) -> BTreeMap<String, Path> {
    let mut parser = Parser::new(source, kpps);
    parser.parse_file()
}
