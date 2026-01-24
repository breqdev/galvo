use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{display::Display, painter::painter};
use vector_apps::{apps::Controls, point::Point};

mod display;
mod painter;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();

    let (tx_path, rx_path): (Sender<Point>, Receiver<Point>) = mpsc::channel();
    let (tx_ctrl, rx_ctrl): (Sender<Controls>, Receiver<Controls>) = mpsc::channel();

    let display = Display::new(rx_path, tx_ctrl);

    thread::spawn(move || {
        painter(tx_path, rx_ctrl);
    });

    eframe::run_native(
        "Galvo Simulator",
        options,
        Box::new(|_cc| Ok(Box::new(display))),
    )
}
