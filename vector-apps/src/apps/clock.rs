use core::f32::consts::PI;

use alloc::format;
use jiff::Timestamp;
use jiff::tz::TimeZone;

use alloc::vec::Vec;
use hershey_text::fonts;

use crate::{
    apps::VectorApp,
    point::{Path, Point},
    utils::text::text_to_path,
};

pub trait TimeSource {
    /// Seconds since Unix epoch (UTC)
    fn now(&self) -> u64;
}

pub struct Clock<T: TimeSource> {
    path: Path,
    time_source: T,
}

impl<T: TimeSource> Clock<T> {
    pub fn new(time_source: T) -> Self {
        Self {
            path: Vec::new(),
            time_source,
        }
    }
}

impl<T: TimeSource> VectorApp for Clock<T> {
    fn get_path(&mut self, frame: u64) -> &Path {
        let color = (
            (libm::sinf(frame as f32 * 0.1) * 127.0 + 128.0) as u8,
            (libm::sinf(frame as f32 * 0.1 + PI * (2.0 / 3.0)) * 127.0 + 128.0) as u8,
            (libm::sinf(frame as f32 * 0.1 + PI * (4.0 / 3.0)) * 127.0 + 128.0) as u8,
        );

        self.path.clear();

        let ts =
            Timestamp::from_second(self.time_source.now() as i64).expect("valid unix timestamp");

        let dt = ts.to_zoned(TimeZone::UTC);

        // let year = dt.year();
        // let month = dt.month() as u8;
        // let day = dt.day() as u8;

        let hour = dt.hour() as u8;
        let minute = dt.minute() as u8;
        let second = dt.second() as u8;

        // Time (HH:MM:SS)
        self.path.append(&mut text_to_path(
            &format!("{:02} {:02} {:02}", hour, minute, second),
            6,
            96,
            1.5,
            1.5,
            color,
            fonts::ROMANS,
        ));

        // Date (YYYY-MM-DD)
        // self.path.append(&mut text_to_path(
        //     &format!("{:04}-{:02}-{:02}", year, month, day),
        //     0,
        //     144,
        //     1.0,
        //     1.0,
        //     color,
        //     fonts::ROMANS,
        // ));

        // draw the lil dots?
        self.path.push(Point {
            x: 78,
            y: 96,
            color: (0, 0, 0),
            delay: 400,
        });
        self.path.push(Point {
            x: 78,
            y: 96,
            color,
            delay: 400,
        });
        self.path.push(Point {
            x: 162,
            y: 96,
            color: (0, 0, 0),
            delay: 400,
        });
        self.path.push(Point {
            x: 162,
            y: 96,
            color,
            delay: 400,
        });

        // laser off at end
        self.path.push(Point {
            x: 192,
            y: 96,
            color: (0, 0, 0),
            delay: 0,
        });

        &self.path
    }
}
