use egui::{
    epaint::QuadraticBezierShape, Color32, Pos2, Rect, Rounding, Sense, Shape, Stroke, Vec2,
};
use emath::RectTransform;
use epaint::PathShape;

const CONTROL_POINT_RADIUS: f32 = 8.0;

pub struct TemplateApp {
    points: Vec<CurvePoint>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            points: vec![
                CurvePoint::First(Pos2::new(0.0, 0.0)),
                CurvePoint::Bezier(Pos2::new(1.0, 25.0)),
                CurvePoint::Inner(Pos2::new(2.0, 50.0)),
                CurvePoint::Bezier(Pos2::new(3.0, 75.0)),
                CurvePoint::Last(Pos2::new(4.0, 100.0)),
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
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let to_screen = emath::RectTransform::from_to(
                Rect::from_min_size(Pos2::ZERO, Vec2::new(4.0, 100.0)),
                Rect::from_min_size(Pos2::ZERO, ui.available_size()),
            );
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());

            for i in 1..=3 {
                painter.add(PathShape::line(
                    vec![
                        to_screen.transform_pos(Pos2::new(i as f32, 0.0)),
                        to_screen.transform_pos(Pos2::new(i as f32, 100.0)),
                    ],
                    Stroke::new(1.0, Color32::GRAY),
                ));
                painter.add(PathShape::line(
                    vec![
                        to_screen.transform_pos(Pos2::new(0.0, (i * 25) as f32)),
                        to_screen.transform_pos(Pos2::new(4.0, (i * 25) as f32)),
                    ],
                    Stroke::new(1.0, Color32::GRAY),
                ));
            }

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

            for i in 0..(points_in_screen.len() - 1) / 2 {
                let shape = QuadraticBezierShape::from_points_stroke(
                    [
                        points_in_screen[i * 2],
                        points_in_screen[i * 2 + 1],
                        points_in_screen[i * 2 + 2],
                    ],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
                );
                painter.add(shape);
            }

            painter.add(PathShape::line(
                points_in_screen,
                Stroke::new(1.0, Color32::RED.linear_multiply(0.25)),
            ));
            painter.extend(control_point_shapes);
        });
    }
}

pub enum CurvePoint {
    First(Pos2),
    /// All other points
    Inner(Pos2),
    /// In between each Outer and Inner Points are the Bezier points to define the curves
    Bezier(Pos2),
    Last(Pos2),
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
            CurvePoint::First(_) => {
                let point_rect = self.point_rect(to_screen);
                Shape::convex_polygon(
                    vec![
                        point_rect.right_top(),
                        point_rect.right_bottom(),
                        point_rect.left_center(),
                    ],
                    Color32::TRANSPARENT,
                    stroke,
                )
            }
            CurvePoint::Inner(_) => {
                Shape::rect_stroke(self.point_rect(to_screen), Rounding::default(), stroke)
            }
            CurvePoint::Bezier(_) => {
                Shape::circle_stroke(self.screen_pos(to_screen), CONTROL_POINT_RADIUS, stroke)
            }
            CurvePoint::Last(_) => {
                let point_rect = self.point_rect(to_screen);
                Shape::convex_polygon(
                    vec![
                        point_rect.left_top(),
                        point_rect.left_bottom(),
                        point_rect.right_center(),
                    ],
                    Color32::TRANSPARENT,
                    stroke,
                )
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
            CurvePoint::First(pos) => *pos,
            CurvePoint::Inner(pos) => *pos,
            CurvePoint::Bezier(pos) => *pos,
            CurvePoint::Last(pos) => *pos,
        }
    }

    pub fn set_pos(&mut self, new_pos: Pos2) {
        match self {
            CurvePoint::First(pos) | CurvePoint::Last(pos) => pos.y = new_pos.y,
            CurvePoint::Inner(pos) | CurvePoint::Bezier(pos) => *pos = new_pos,
        }
    }
}
