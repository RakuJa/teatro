use crate::gui::icons::{self, Icon};
use eframe::emath::Rect;
use eframe::epaint::Color32;

pub struct ButtonStyle {
    bg_color: Color32,
    border_color: Color32,
    border_width: f32,
    corner_radius: f32,
}

impl ButtonStyle {
    pub(crate) fn new(scale: f32) -> Self {
        Self {
            bg_color: Color32::from_rgb(60, 60, 70),
            border_color: Color32::from_rgb(100, 100, 110),
            border_width: 1.5 * scale,
            corner_radius: 4.0 * scale,
        }
    }

    pub(crate) const fn active_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }
}

pub fn draw_button(
    ui: &egui::Ui,
    rect: Rect,
    style: &ButtonStyle,
    icon: Icon,
    icon_size: f32,
    id: &str,
) -> egui::Response {
    ui.painter()
        .rect_filled(rect, style.corner_radius, style.bg_color);
    ui.painter().rect_stroke(
        rect,
        style.corner_radius,
        egui::Stroke::new(style.border_width, style.border_color),
        egui::StrokeKind::Outside,
    );
    icons::paint(
        ui.painter(),
        icon,
        Rect::from_center_size(rect.center(), egui::Vec2::splat(icon_size)),
        Color32::from_rgb(240, 240, 250),
    );

    ui.interact(rect, ui.id().with(id), egui::Sense::click())
}
