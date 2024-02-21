mod curve;

use self::curve::Curve;
use egui::Slider;

#[derive(Default)]
pub struct TemplateApp {
    show_progress: bool,
    x: f32,
    curve: Curve,
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.show_progress, "Show values");
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
