use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
    time::{Duration, Instant},
};

use egui::{Color32, Pos2};
use iterslide::SlideIterator;

use vector_apps::{apps::Controls, point::Point};

struct TrailPoint {
    pos: Point,
    ts: Instant,
}

pub struct Display {
    trail: VecDeque<TrailPoint>,
    rx: Receiver<Point>,
    tx: Sender<Controls>,
}

impl Display {
    pub fn new(rx: Receiver<Point>, tx: Sender<Controls>) -> Self {
        Self {
            trail: VecDeque::new(),
            rx,
            tx,
        }
    }
}

fn to_screen(pos: Point, center: Pos2, scale: f32) -> Pos2 {
    Pos2 {
        x: center.x + (pos.x as f32 - 128.0) / 255.0 * scale,
        y: center.y + (pos.y as f32 - 128.0) / 255.0 * scale,
    }
}

impl eframe::App for Display {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();

        while let Ok(point) = self.rx.try_recv() {
            self.trail.push_front(TrailPoint {
                pos: point,
                ts: now,
            });
        }

        loop {
            match self.trail.back() {
                Some(point) if point.ts < now - Duration::from_millis(200) => {
                    self.trail.pop_back().unwrap();
                }
                _ => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Galvo Simulator");

            // Build controls state to send to application

            let ctrl_x = if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                -1
            } else {
                0
            } + if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                1
            } else {
                0
            };

            let ctrl_y = if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                -1
            } else {
                0
            } + if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                1
            } else {
                0
            };

            let a = ctx.input(|i| i.key_pressed(egui::Key::Space));
            let b = ctx.input(|i| i.key_pressed(egui::Key::Enter));

            let controls = Controls {
                x: ctrl_x,
                y: ctrl_y,
                a,
                b,
            };
            self.tx.send(controls).unwrap();

            // Render application

            let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

            ui.painter().rect_filled(rect, 0.0, Color32::BLACK);

            let center = rect.center();
            let scale = rect.width().min(rect.height());

            let painter = ui.painter();

            for window in self.trail.iter().slide(2) {
                let a = window[0];
                let b = window[1];
                let age = now.duration_since(b.ts);

                let alpha = ((1.0 - (age.as_secs_f32() / 0.1)) * 255.0) as u8;

                let color = Color32::from_rgba_unmultiplied(
                    a.pos.color.0,
                    a.pos.color.1,
                    a.pos.color.2,
                    alpha,
                );

                if color.to_opaque() != Color32::BLACK {
                    // skip blank traces
                    painter.line_segment(
                        [
                            to_screen(a.pos, center, scale),
                            to_screen(b.pos, center, scale),
                        ],
                        egui::Stroke::new(1.0, color),
                    );
                }
            }
        });

        ctx.request_repaint();
    }
}
