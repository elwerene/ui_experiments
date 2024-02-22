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
    edit_mode: bool,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            edit_mode: false,
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

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_progress, "Values");
                ui.add_enabled(
                    self.show_progress,
                    Checkbox::new(&mut self.run, "Run with 120bpm"),
                );
                ui.add_enabled(
                    self.show_progress,
                    Slider::new(&mut self.x, 0.0f32..=4.0f32),
                );
                ui.checkbox(&mut self.edit_mode, "Edit mode");
                ui.menu_button("Examples", |ui| {
                    if ui.button("Forward").clicked() {
                        self.curve = Curve::forward();
                        ui.close_menu();
                    }
                    if ui.button("Backward").clicked() {
                        self.curve = Curve::backward();
                        ui.close_menu();
                    }
                    if ui.button("Alternating").clicked() {
                        self.curve = Curve::alternating();
                        ui.close_menu();
                    }
                    if ui.button("Fixed").clicked() {
                        self.curve = Curve::fixed();
                        ui.close_menu();
                    }
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.curve.draw(
                ui,
                Some(self.x).filter(|_| self.show_progress),
                self.edit_mode,
            );
        });
    }
}
