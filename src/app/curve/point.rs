use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use emath::RectTransform;

const CONTROL_POINT_RADIUS: f32 = 8.0;

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
