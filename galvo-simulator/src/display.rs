use std::{
    collections::VecDeque,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use egui::{Color32, Pos2};
use iterslide::SlideIterator;

use vector_apps::point::Point;

struct TrailPoint {
    pos: Point,
    ts: Instant,
}

pub struct Display {
    trail: VecDeque<TrailPoint>,
    receiver: Receiver<Point>,
}

impl Display {
    pub fn new(receiver: Receiver<Point>) -> Self {
        Self {
            trail: VecDeque::new(),
            receiver,
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

        while let Ok(point) = self.receiver.try_recv() {
            self.trail.push_front(TrailPoint {
                pos: point,
                ts: now,
            });
        }

        loop {
            match self.trail.back() {
                Some(point) if point.ts < now - Duration::from_millis(10) => {
                    self.trail.pop_back().unwrap();
                }
                _ => break,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.heading("Galvo Simulator");

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

                let color = if a.pos.color > 0 {
                    Color32::from_rgba_unmultiplied(255, 0, 0, alpha)
                } else {
                    Color32::from_rgba_unmultiplied(127, 127, 127, alpha / 2)
                };

                painter.line_segment(
                    [
                        to_screen(a.pos, center, scale),
                        to_screen(b.pos, center, scale),
                    ],
                    egui::Stroke::new(1.0, color),
                );
            }
        });

        ctx.request_repaint();
    }
}
