use egui::{
    epaint::QuadraticBezierShape, Color32, Pos2, Rect, Rounding, Sense, Shape, Stroke, Vec2,
};
use emath::RectTransform;
use epaint::PathShape;

const CONTROL_POINT_RADIUS: f32 = 8.0;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    points: Vec<CurvePoint>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            points: vec![
                CurvePoint::Outer(Pos2::new(0.0, 0.0)),
                CurvePoint::Bezier(Pos2::new(2.0, 50.0)),
                CurvePoint::Outer(Pos2::new(4.0, 100.0)),
            ],
        }
    }
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for TemplateApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, Vec2::new(4.0, 100.0)),
                Rect::from_min_size(Pos2::ZERO, ui.available_size()),
            );

            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());

            let control_point_shapes: Vec<Shape> = self
                .points
                .iter_mut()
                .enumerate()
                .map(|(i, point)| {
                    let point_id = response.id.with(i);
                    let point_response =
                        ui.interact(point.point_rect(to_screen), point_id, Sense::drag());
                    point.set_screen_pos(
                        to_screen,
                        point.screen_pos(to_screen) + point_response.drag_delta(),
                    );
                    let stroke = ui.style().interact(&point_response).fg_stroke;
                    point.shape(to_screen, stroke)
                })
                .collect();

            let points_in_screen: Vec<Pos2> = self
                .points
                .iter()
                .map(|p| p.screen_pos(to_screen))
                .collect();
            let points = points_in_screen.clone().try_into().unwrap();
            let shape = QuadraticBezierShape::from_points_stroke(
                points,
                false,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            );
            painter.add(shape);
            painter.add(PathShape::line(
                points_in_screen,
                Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            ));
            painter.extend(control_point_shapes);
        });
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum CurvePoint {
    /// First and last must be a outer point
    Outer(Pos2),
    /// All other points
    Inner(Pos2),
    /// In between each Outer and Inner Points are the Bezier points to define the curves
    Bezier(Pos2),
}

impl CurvePoint {
    pub fn point_rect(&self, to_screen: RectTransform) -> Rect {
        Rect::from_center_size(
            self.screen_pos(to_screen),
            Vec2::splat(2.0 * CONTROL_POINT_RADIUS),
        )
    }

    pub fn shape(&self, to_screen: RectTransform, stroke: Stroke) -> Shape {
        match self {
            CurvePoint::Outer(_) => {
                Shape::rect_stroke(self.point_rect(to_screen), Rounding::default(), stroke)
            }
            CurvePoint::Inner(_) => todo!(),
            CurvePoint::Bezier(_) => {
                Shape::circle_stroke(self.screen_pos(to_screen), CONTROL_POINT_RADIUS, stroke)
            }
        }
    }

    pub fn screen_pos(&self, to_screen: RectTransform) -> Pos2 {
        to_screen.transform_pos(self.pos())
    }

    pub fn set_screen_pos(&mut self, to_screen: RectTransform, screen_pos: Pos2) {
        self.set_pos(to_screen.inverse().transform_pos_clamped(screen_pos));
    }

    pub fn pos(&self) -> Pos2 {
        match self {
            CurvePoint::Outer(pos) => *pos,
            CurvePoint::Inner(pos) => *pos,
            CurvePoint::Bezier(pos) => *pos,
        }
    }

    pub fn set_pos(&mut self, new_pos: Pos2) {
        match self {
            CurvePoint::Outer(pos) => pos.y = new_pos.y,
            CurvePoint::Inner(pos) => *pos = new_pos,
            CurvePoint::Bezier(pos) => *pos = new_pos,
        }
    }
}
