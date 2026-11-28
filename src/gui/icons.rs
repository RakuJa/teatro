use egui::{Color32, Painter, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Keys,
    #[cfg_attr(
        not(feature = "bybe"),
        allow(dead_code, reason = "only the BYBE tab uses it")
    )]
    Globe,
    Gear,
    Document,
    Check,
    Warning,
    Info,
    Speaker,
    Headphones,
    ArrowUp,
    ArrowDown,
    Trash,
    List,
    Shuffle,
    SpeakerMuted,
    Solo,
    Stop,
    Play,
    Pause,
    Next,
}

struct Canvas {
    rect: Rect,
    stroke: Stroke,
    color: Color32,
}

impl Canvas {
    fn p(&self, x: f32, y: f32) -> Pos2 {
        self.rect.min + Vec2::new(x, y) * self.rect.width()
    }

    fn len(&self, v: f32) -> f32 {
        v * self.rect.width()
    }

    fn line(&self, painter: &Painter, points: &[(f32, f32)]) {
        let points = points.iter().map(|&(x, y)| self.p(x, y)).collect();
        painter.add(Shape::line(points, self.stroke));
    }

    fn closed(&self, painter: &Painter, points: &[(f32, f32)]) {
        let points = points.iter().map(|&(x, y)| self.p(x, y)).collect();
        painter.add(Shape::closed_line(points, self.stroke));
    }

    fn filled(&self, painter: &Painter, points: &[(f32, f32)]) {
        let points = points.iter().map(|&(x, y)| self.p(x, y)).collect();
        painter.add(Shape::convex_polygon(points, self.color, Stroke::NONE));
    }

    fn rect_stroke(&self, painter: &Painter, min: (f32, f32), max: (f32, f32), rounding: f32) {
        painter.rect_stroke(
            Rect::from_min_max(self.p(min.0, min.1), self.p(max.0, max.1)),
            self.len(rounding),
            self.stroke,
            egui::StrokeKind::Middle,
        );
    }

    fn rect_filled(&self, painter: &Painter, min: (f32, f32), max: (f32, f32), rounding: f32) {
        painter.rect_filled(
            Rect::from_min_max(self.p(min.0, min.1), self.p(max.0, max.1)),
            self.len(rounding),
            self.color,
        );
    }

    /// Elliptical arc around `center` (unit coords), angles in radians.
    fn arc(&self, painter: &Painter, center: (f32, f32), radii: (f32, f32), from: f32, to: f32) {
        const SEGMENTS: usize = 24;
        let points = (0..=SEGMENTS)
            .map(|i| {
                let a = (to - from).mul_add(i as f32 / SEGMENTS as f32, from);
                self.p(
                    radii.0.mul_add(a.cos(), center.0),
                    radii.1.mul_add(a.sin(), center.1),
                )
            })
            .collect();
        painter.add(Shape::line(points, self.stroke));
    }
}

/// Paints `icon` centered in `rect` (drawn in its largest inscribed square).
pub fn paint(painter: &Painter, icon: Icon, rect: Rect, color: Color32) {
    let side = rect.width().min(rect.height());
    let square = Rect::from_center_size(rect.center(), Vec2::splat(side));
    let c = Canvas {
        rect: square,
        stroke: Stroke::new((side * 0.09).max(1.2), color),
        color,
    };
    match icon {
        Icon::Keys => paint_keys(painter, &c),
        Icon::Globe => {
            c.arc(painter, (0.5, 0.5), (0.4, 0.4), 0.0, TAU);
            c.arc(painter, (0.5, 0.5), (0.17, 0.4), 0.0, TAU);
            c.line(painter, &[(0.1, 0.5), (0.9, 0.5)]);
        }
        Icon::Gear => paint_gear(painter, &c),
        Icon::Document => {
            c.closed(
                painter,
                &[
                    (0.22, 0.1),
                    (0.62, 0.1),
                    (0.8, 0.28),
                    (0.8, 0.9),
                    (0.22, 0.9),
                ],
            );
            for y in [0.42, 0.58, 0.74] {
                c.line(painter, &[(0.34, y), (0.68, y)]);
            }
        }
        Icon::Check => c.line(painter, &[(0.16, 0.54), (0.4, 0.76), (0.86, 0.26)]),
        Icon::Warning => {
            c.closed(painter, &[(0.5, 0.1), (0.92, 0.86), (0.08, 0.86)]);
            c.line(painter, &[(0.5, 0.38), (0.5, 0.6)]);
            painter.circle_filled(c.p(0.5, 0.73), c.len(0.055), color);
        }
        Icon::Info => {
            c.arc(painter, (0.5, 0.5), (0.4, 0.4), 0.0, TAU);
            c.line(painter, &[(0.5, 0.45), (0.5, 0.72)]);
            painter.circle_filled(c.p(0.5, 0.31), c.len(0.055), color);
        }
        Icon::Speaker | Icon::SpeakerMuted => {
            paint_speaker(painter, &c, icon == Icon::SpeakerMuted);
        }
        Icon::Headphones => {
            c.arc(painter, (0.5, 0.58), (0.36, 0.42), PI, TAU);
            c.rect_filled(painter, (0.08, 0.5), (0.36, 0.9), 0.08);
            c.rect_filled(painter, (0.64, 0.5), (0.92, 0.9), 0.08);
        }
        Icon::ArrowUp => c.filled(painter, &[(0.5, 0.25), (0.82, 0.68), (0.18, 0.68)]),
        Icon::ArrowDown => c.filled(painter, &[(0.18, 0.32), (0.82, 0.32), (0.5, 0.75)]),
        Icon::Trash => {
            c.line(painter, &[(0.14, 0.24), (0.86, 0.24)]);
            c.line(
                painter,
                &[(0.38, 0.24), (0.4, 0.12), (0.6, 0.12), (0.62, 0.24)],
            );
            c.closed(
                painter,
                &[(0.24, 0.32), (0.76, 0.32), (0.7, 0.9), (0.3, 0.9)],
            );
            c.line(painter, &[(0.44, 0.44), (0.44, 0.78)]);
            c.line(painter, &[(0.56, 0.44), (0.56, 0.78)]);
        }
        Icon::List => {
            for y in [0.28, 0.5, 0.72] {
                c.line(painter, &[(0.18, y), (0.82, y)]);
            }
        }
        Icon::Shuffle => {
            c.line(painter, &[(0.104, 0.271), (0.771, 0.729)]);
            c.line(painter, &[(0.104, 0.729), (0.771, 0.271)]);
            c.filled(painter, &[(0.979, 0.729), (0.729, 0.554), (0.729, 0.904)]);
            c.filled(painter, &[(0.979, 0.271), (0.729, 0.096), (0.729, 0.446)]);
        }
        Icon::Solo => {
            // Upper bowl: from the top right, over the top, down to the middle.
            c.arc(painter, (0.5, 0.34), (0.16, 0.16), -0.35, -1.5 * PI);
            // Lower bowl: from the middle, round the right, to the bottom left.
            c.arc(painter, (0.5, 0.66), (0.16, 0.16), -FRAC_PI_2, 2.8);
        }
        Icon::Stop => c.rect_filled(painter, (0.219, 0.219), (0.781, 0.781), 0.04),
        Icon::Play => c.filled(painter, &[(0.312, 0.188), (0.312, 0.812), (0.833, 0.5)]),
        Icon::Pause => {
            c.rect_filled(painter, (0.238, 0.188), (0.412, 0.812), 0.03);
            c.rect_filled(painter, (0.588, 0.188), (0.762, 0.812), 0.03);
        }
        Icon::Next => {
            c.filled(painter, &[(0.188, 0.229), (0.188, 0.771), (0.646, 0.5)]);
            c.rect_filled(painter, (0.688, 0.229), (0.833, 0.771), 0.02);
        }
    }
}

fn paint_speaker(painter: &Painter, c: &Canvas, muted: bool) {
    c.rect_filled(painter, (0.125, 0.354), (0.333, 0.646), 0.02);
    c.filled(
        painter,
        &[
            (0.333, 0.354),
            (0.583, 0.146),
            (0.583, 0.854),
            (0.333, 0.646),
        ],
    );
    if muted {
        c.line(painter, &[(0.708, 0.354), (0.98, 0.646)]);
        c.line(painter, &[(0.708, 0.646), (0.98, 0.354)]);
    } else {
        let spread = 42.0f32.to_radians();
        c.arc(painter, (0.583, 0.5), (0.25, 0.25), -spread, spread);
        c.arc(painter, (0.583, 0.5), (0.4375, 0.4375), -spread, spread);
    }
}

fn paint_keys(painter: &Painter, c: &Canvas) {
    c.rect_stroke(painter, (0.1, 0.18), (0.9, 0.82), 0.06);
    for x in [0.37, 0.63] {
        c.line(painter, &[(x, 0.56), (x, 0.82)]);
    }
    for x in [0.37, 0.63] {
        c.rect_filled(painter, (x - 0.08, 0.18), (x + 0.08, 0.56), 0.02);
    }
}

fn paint_gear(painter: &Painter, c: &Canvas) {
    const TEETH: usize = 8;
    let (outer, inner): (f32, f32) = (0.44, 0.31);
    // Each tooth spans a quarter of its sector at the tip, half at the root.
    let step = TAU / TEETH as f32;
    let mut profile = Vec::with_capacity(TEETH * 4);
    for i in 0..TEETH {
        let a = (i as f32).mul_add(step, -FRAC_PI_2);
        for (da, r) in [
            (-step * 0.25, inner),
            (-step * 0.14, outer),
            (step * 0.14, outer),
            (step * 0.25, inner),
        ] {
            profile.push((
                r.mul_add((a + da).cos(), 0.5),
                r.mul_add((a + da).sin(), 0.5),
            ));
        }
    }
    c.closed(painter, &profile);
    c.arc(painter, (0.5, 0.5), (0.12, 0.12), 0.0, TAU);
}

/// Allocates a `size`-sized square and paints `icon` in it.
pub fn icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), icon, rect, color);
    }
    response
}

/// Icon followed by text, both in `color`. Text is sized `size` points and
/// wraps; the icon stays aligned with the first line.
pub fn icon_label(ui: &mut Ui, icon: Icon, text: impl Into<String>, size: f32, color: Color32) {
    let row_height = ui.fonts_mut(|f| f.row_height(&egui::FontId::proportional(size)));
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = size * 0.4;
        let label = egui::Label::new(RichText::new(text.into()).size(size).color(color)).wrap();
        let paint_icon = |ui: &mut Ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(size, row_height), Sense::hover());
            paint(ui.painter(), icon, rect, color);
        };
        if ui.layout().prefer_right_to_left() {
            ui.add(label);
            paint_icon(ui);
        } else {
            paint_icon(ui);
            ui.add(label);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    const ALL: [Icon; 20] = [
        Icon::Keys,
        Icon::Globe,
        Icon::Gear,
        Icon::Document,
        Icon::Check,
        Icon::Warning,
        Icon::Info,
        Icon::Speaker,
        Icon::Headphones,
        Icon::ArrowUp,
        Icon::ArrowDown,
        Icon::Trash,
        Icon::List,
        Icon::Shuffle,
        Icon::SpeakerMuted,
        Icon::Solo,
        Icon::Stop,
        Icon::Play,
        Icon::Pause,
        Icon::Next,
    ];

    #[expect(dead_code, reason = "compile-time check only")]
    const fn exhaustive(icon: Icon) {
        match icon {
            Icon::Keys
            | Icon::Globe
            | Icon::Gear
            | Icon::Document
            | Icon::Check
            | Icon::Warning
            | Icon::Info
            | Icon::Speaker
            | Icon::Headphones
            | Icon::ArrowUp
            | Icon::ArrowDown
            | Icon::Trash
            | Icon::List
            | Icon::Shuffle
            | Icon::SpeakerMuted
            | Icon::Solo
            | Icon::Stop
            | Icon::Play
            | Icon::Pause
            | Icon::Next => {}
        }
    }

    fn frame_shapes(icon: Option<Icon>, rect: Rect) -> Vec<egui::epaint::ClippedShape> {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            if let Some(icon) = icon {
                paint(ui.painter(), icon, rect, Color32::WHITE);
            }
        });
        let shapes = std::mem::take(&mut output.shapes);
        output.drop_without_applying_deltas();
        shapes
    }

    #[test]
    fn every_icon_paints_inside_its_square() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(16.0));
        let baseline = frame_shapes(None, rect).len();
        for icon in ALL {
            let shapes = frame_shapes(Some(icon), rect);
            assert!(shapes.len() > baseline, "{icon:?} drew nothing");
            for shape in &shapes[baseline..] {
                let bounds = shape.shape.visual_bounding_rect();
                assert!(
                    rect.expand(1.5).contains_rect(bounds),
                    "{icon:?} overflows: {bounds:?}"
                );
            }
        }
    }
}
