mod curve;

use self::curve::Curve;
use chrono::{NaiveDateTime, Utc};
use egui::{Checkbox, Slider};

pub struct TemplateApp {
    show_progress: bool,
    run: bool,
    start: NaiveDateTime,
    x: f32,
    curve: Curve,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            show_progress: false,
            run: false,
            start: Utc::now().naive_utc(),
            x: Default::default(),
            curve: Default::default(),
        }
    }
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.run {
            self.x = (((Utc::now().naive_utc() - self.start).num_milliseconds() as f64 / 500.0)
                % 4.0) as f32;
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_progress, "Show values");
                ui.add_enabled(
                    self.show_progress,
                    Checkbox::new(&mut self.run, "Run with 120bpm"),
                );
                ui.add_enabled(
                    self.show_progress,
                    Slider::new(&mut self.x, 0.0f32..=4.0f32),
                );
            });

            self.curve
                .draw(ui, Some(self.x).filter(|_| self.show_progress));
        });
    }
}
