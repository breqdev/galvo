use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{display::Display, painter::painter};
use vector_apps::point::Point;

mod display;
mod painter;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    let (tx, rx): (Sender<Point>, Receiver<Point>) = mpsc::channel();
    let display = Display::new(rx);

    thread::spawn(move || {
        painter(tx);
    });

    eframe::run_native(
        "Galvo Simulator",
        options,
        Box::new(|_cc| Ok(Box::new(display))),
    )
}
