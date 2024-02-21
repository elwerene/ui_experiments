use egui::{
    epaint::QuadraticBezierShape, Color32, Pos2, Rect, Rounding, Sense, Shape, Stroke, Vec2,
};
use emath::RectTransform;
use epaint::PathShape;

const CONTROL_POINT_RADIUS: f32 = 8.0;

pub struct TemplateApp {
    linked: bool,
    points: Vec<CurvePoint>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            linked: false,
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
                Rect::from_min_size(ui.next_widget_position(), ui.available_size()),
            );

            let mut outer_change = None;
            let mut bezier_to_line = None;
            let mut remove_point = None;

            let (response, painter) = ui.allocate_painter(to_screen.to().size(), Sense::hover());
            let new_point_id = response.id.with(self.points.len());
            let new_point_response = ui.interact(*to_screen.to(), new_point_id, Sense::click());
            if new_point_response.double_clicked()
                || new_point_response.clicked_by(egui::PointerButton::Secondary)
            {
                if let Some((pos, i, before, after)) =
                    new_point_response.interact_pointer_pos().and_then(|pos| {
                        self.points
                            .iter()
                            .position(|point| point.screen_pos(to_screen).x > pos.x)
                            .and_then(|i| {
                                self.points.get(i).and_then(|after| {
                                    self.points
                                        .get(i - 1)
                                        .map(|before| (pos, i, *before, *after))
                                })
                            })
                    })
                {
                    if before.is_inner() || before.is_outer() {
                        self.points
                            .insert(i, CurvePoint::Inner(to_screen.inverse().transform_pos(pos)));
                        self.points.insert(
                            i,
                            CurvePoint::Bezier(to_screen.inverse().transform_pos(Pos2::new(
                                (before.screen_pos(to_screen).x + pos.x) / 2.0,
                                (before.screen_pos(to_screen).y + pos.y) / 2.0,
                            ))),
                        );
                        bezier_to_line = Some(i + 2);
                    } else {
                        bezier_to_line = Some(i - 1);
                        self.points.insert(
                            i,
                            CurvePoint::Bezier(to_screen.inverse().transform_pos(Pos2::new(
                                (after.screen_pos(to_screen).x + pos.x) / 2.0,
                                (after.screen_pos(to_screen).y + pos.y) / 2.0,
                            ))),
                        );
                        self.points
                            .insert(i, CurvePoint::Inner(to_screen.inverse().transform_pos(pos)));
                    }
                }
            }

            for i in 0..=4 {
                let stroke = Stroke::new(
                    match i {
                        0 | 4 => 1.0,
                        _ => 0.5,
                    },
                    Color32::GRAY,
                );
                painter.add(PathShape::line(
                    vec![
                        to_screen.transform_pos(Pos2::new(i as f32, 0.0)),
                        to_screen.transform_pos(Pos2::new(i as f32, 100.0)),
                    ],
                    stroke,
                ));
                painter.add(PathShape::line(
                    vec![
                        to_screen.transform_pos(Pos2::new(0.0, (i * 25) as f32)),
                        to_screen.transform_pos(Pos2::new(4.0, (i * 25) as f32)),
                    ],
                    stroke,
                ));
            }

            let x_limits = { 0..self.points.len() }
                .map(|i| {
                    if i == 0 {
                        return None;
                    }
                    self.points
                        .get(i - 1)
                        .map(|point| point.screen_pos(to_screen).x)
                        .and_then(|before| {
                            self.points
                                .get(i + 1)
                                .map(|point| (before, point.screen_pos(to_screen).x))
                        })
                })
                .collect::<Vec<_>>();

            let control_point_shapes: Vec<Shape> = self
                .points
                .iter_mut()
                .enumerate()
                .map(|(i, point)| {
                    let point_id = response.id.with(i);
                    let point_response = ui.interact(
                        point.point_rect(to_screen),
                        point_id,
                        Sense::click_and_drag(),
                    );

                    if point_response.double_clicked()
                        || point_response.clicked_by(egui::PointerButton::Secondary)
                    {
                        if point.is_outer() {
                            self.linked = !self.linked;
                            if self.linked {
                                outer_change = Some((i, point.pos()));
                            }
                        } else if point.is_bezier() {
                            bezier_to_line = Some(i);
                        } else if point.is_inner() {
                            remove_point = Some(i);
                        }
                    } else {
                        let mut new_screen_pos =
                            point.screen_pos(to_screen) + point_response.drag_delta();
                        if let Some(x_limit) = x_limits.get(i).and_then(|x_limit| *x_limit) {
                            new_screen_pos.x = new_screen_pos.x.clamp(x_limit.0, x_limit.1)
                        }
                        point.set_screen_pos(to_screen, new_screen_pos);
                        if point_response.dragged() && point.is_outer() && self.linked {
                            outer_change = Some((i, point.pos()));
                        }
                    }

                    let stroke = if point.is_outer() && self.linked {
                        let mut stroke = ui.style().interact(&point_response).fg_stroke;
                        stroke.color = Color32::LIGHT_BLUE;
                        stroke
                    } else {
                        ui.style().interact(&point_response).fg_stroke
                    };
                    point.shape(to_screen, stroke)
                })
                .collect();

            if let Some((i, pos)) = outer_change {
                let i = self.points.len() - i - 1;
                if let Some(point) = self.points.get_mut(i) {
                    point.set_pos(pos);
                }
                ctx.request_repaint();
            }
            if let Some(i) = remove_point {
                self.points.remove(i);
                self.points.remove(i);
                bezier_to_line = Some(i - 1);
            }
            if let Some(i) = bezier_to_line {
                if let (Some(before), Some(after), Some(point)) = (
                    self.points.get(i - 1).map(|point| point.pos()),
                    self.points.get(i + 1).map(|point| point.pos()),
                    self.points.get_mut(i),
                ) {
                    point.set_pos(Pos2::new(
                        (before.x + after.x) / 2.0,
                        (before.y + after.y) / 2.0,
                    ));
                }
            }

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

#[derive(Copy, Clone, Debug)]
pub enum CurvePoint {
    First(Pos2),
    /// All other points
    Inner(Pos2),
    /// In between each Outer and Inner Points are the Bezier points to define the curves
    Bezier(Pos2),
    Last(Pos2),
}

impl CurvePoint {
    pub fn is_inner(&self) -> bool {
        matches!(self, CurvePoint::Inner(_))
    }

    pub fn is_outer(&self) -> bool {
        matches!(self, CurvePoint::First(_) | CurvePoint::Last(_))
    }

    pub fn is_bezier(&self) -> bool {
        matches!(self, CurvePoint::Bezier(_))
    }

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
                        point_rect.center_top(),
                        point_rect.center_bottom(),
                        point_rect.right_center(),
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
                        point_rect.center_top(),
                        point_rect.center_bottom(),
                        point_rect.left_center(),
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
