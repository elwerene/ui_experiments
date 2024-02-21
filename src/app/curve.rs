mod point;

use self::point::CurvePoint;
use egui::{epaint::QuadraticBezierShape, Color32, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use epaint::PathShape;

pub struct Curve {
    linked: bool,
    points: Vec<CurvePoint>,
}

impl Default for Curve {
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

impl Curve {
    pub fn draw(&mut self, ui: &mut Ui, beat_position: Option<f32>) {
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

        // draw current position
        if let Some(beat_position) = beat_position {
            painter.add(PathShape::line(
                vec![
                    to_screen.transform_pos(Pos2::new(beat_position, 0.0)),
                    to_screen.transform_pos(Pos2::new(beat_position, 100.0)),
                ],
                Stroke::new(1.0, Color32::from_rgb(160, 0, 150)),
            ));

            let y = 100.0 - self.value(beat_position);
            painter.add(PathShape::line(
                vec![
                    to_screen.transform_pos(Pos2::new(0.0, y)),
                    to_screen.transform_pos(Pos2::new(4.0, y)),
                ],
                Stroke::new(1.0, Color32::from_rgb(160, 0, 150)),
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
            ui.ctx().request_repaint();
        }
        if let Some(i) = remove_point {
            self.points.remove(i);
            self.points.remove(i);
            bezier_to_line = Some(i - 1);
            ui.ctx().request_repaint();
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
            ui.ctx().request_repaint();
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
    }

    pub fn value(&self, beat_position: f32) -> f32 {
        if !(0.0..=4.0).contains(&beat_position) {
            panic!("Beat position out of range 0.0..=4.0: {beat_position}");
        }

        let y = self
            .points
            .iter()
            .position(|point| point.pos().x > beat_position)
            .and_then(|i| {
                self.points.get(i).and_then(|point| {
                    if point.is_bezier() {
                        self.points.get(i - 1).and_then(|start| {
                            self.points
                                .get(i + 1)
                                .map(|end| (start.pos(), point.pos(), end.pos()))
                        })
                    } else {
                        self.points.get(i - 1).and_then(|bezier| {
                            self.points
                                .get(i - 2)
                                .map(|start| (start.pos(), bezier.pos(), point.pos()))
                        })
                    }
                })
            })
            .map(|(start, bezier, end)| {
                let bezier = QuadraticBezierShape::from_points_stroke(
                    [start, bezier, end],
                    false,
                    Color32::TRANSPARENT,
                    Stroke::default(),
                );
                let mut n = 2;
                let mut t = 0.5;
                loop {
                    let sample = bezier.sample(t);
                    let e = sample.x - beat_position;
                    if e.abs() < 0.001 {
                        break sample.y;
                    }
                    let step = 1.0 / 2.0f32.powi(n);
                    if e < 0.0 {
                        t += step;
                    } else {
                        t -= step;
                    }
                    n += 1;
                }
            })
            .unwrap_or_else(|| {
                self.points
                    .last()
                    .expect("Could not get last point")
                    .pos()
                    .y
            });
        100.0 - y
    }
}
