use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player::player::BAR_HEIGHT;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;
use egui::{CursorIcon, Id, Sense, Stroke, StrokeKind};

const BAR_HIT_MARGIN: f32 = 8.0;
/// How long to keep showing the seek target while the backend catches up.
const PENDING_SEEK_SECS: f64 = 0.4;
const HOVER_ANIM_SECS: f32 = 0.12;

mod palette {
    use eframe::epaint::Color32;

    pub const TRACK: Color32 = Color32::from_rgb(35, 35, 45);
    pub const BORDER_IDLE: Color32 = Color32::from_rgb(55, 55, 65);
    pub const BORDER_HOVER: Color32 = Color32::from_rgb(95, 80, 115);
    pub const FILL_IDLE: Color32 = Color32::from_rgb(150, 70, 190);
    pub const FILL_HOVER: Color32 = Color32::from_rgb(170, 90, 210);
    pub const KNOB: Color32 = Color32::from_rgb(230, 180, 255);
    pub const KNOB_GLOW: Color32 = Color32::from_rgb(200, 120, 240);
}

impl AkaiVisualizer {
    /// Draws the progress bar and handles seeking: a click or the end of a
    /// drag sends `CommsCommand::SeekTo` with a position in `0.0..=1.0`.
    pub(crate) fn draw_progress_bar(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        progress: f32,
    ) {
        let progress = sanitize_progress(progress);

        let bar_height = BAR_HEIGHT * scale;
        let radius = bar_height / 2.0;
        let bar_rect = Rect::from_min_size(
            Pos2::new(
                player_rect.min.x + content_padding,
                player_rect.max.y - content_padding - bar_height,
            ),
            Vec2::new(
                2.0f32
                    .mul_add(-content_padding, player_rect.width())
                    .max(0.0),
                bar_height,
            ),
        );

        let hit_rect = bar_rect.expand2(Vec2::new(0.0, BAR_HIT_MARGIN * scale));
        let id = Id::new("akai_progress_bar");
        let response = ui.interact(hit_rect, id, Sense::click_and_drag());

        let pos_to_progress = |x: f32| {
            if bar_rect.width() > 0.0 {
                ((x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };

        let dragging = response.dragged();
        let hovered = response.hovered() || dragging;
        if hovered {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        let drag_id = id.with("drag_target");
        let pending_id = id.with("pending_seek");
        let now = ui.input(|i| i.time);

        let drag_pos = if dragging {
            response
                .interact_pointer_pos()
                .map(|p| pos_to_progress(p.x))
        } else {
            None
        };
        if let Some(p) = drag_pos {
            ui.ctx().data_mut(|d| d.insert_temp(drag_id, p));
        }

        let committed = if response.drag_stopped() {
            ui.ctx().data(|d| d.get_temp::<f32>(drag_id))
        } else if response.clicked() {
            response
                .interact_pointer_pos()
                .map(|p| pos_to_progress(p.x))
        } else {
            None
        };

        if let Some(target) = committed {
            self.send_command_to_backend(CommsCommand::SeekTo { target });
            ui.ctx()
                .data_mut(|d| d.insert_temp(pending_id, (target, now)));
        }

        // The backend's progress lags after a seek, so keep showing the target
        // briefly to stop the knob snapping back.
        let pending = ui
            .ctx()
            .data(|d| d.get_temp::<(f32, f64)>(pending_id))
            .filter(|&(_, t)| now - t < PENDING_SEEK_SECS)
            .map(|(target, _)| target);
        if pending.is_some() {
            ui.ctx().request_repaint();
        }

        let shown = drag_pos.or(pending).unwrap_or(progress);

        let hover_t = ui
            .ctx()
            .animate_bool_with_time(id.with("hover"), hovered, HOVER_ANIM_SECS);

        let painter = ui.painter();
        painter.rect_filled(bar_rect, radius, palette::TRACK);

        // Hover preview: dim fill from the playhead to the cursor.
        if let Some(hover_pos) = response.hover_pos().filter(|_| !dragging) {
            let hover_x = bar_rect
                .width()
                .mul_add(pos_to_progress(hover_pos.x), bar_rect.min.x);
            let played_x = bar_rect.width().mul_add(shown, bar_rect.min.x);
            if hover_x > played_x {
                painter.rect_filled(
                    Rect::from_min_max(
                        Pos2::new(played_x, bar_rect.min.y),
                        Pos2::new(hover_x, bar_rect.max.y),
                    ),
                    radius,
                    palette::FILL_IDLE.gamma_multiply(0.25),
                );
            }
        }

        let border = palette::BORDER_IDLE.lerp_to_gamma(palette::BORDER_HOVER, hover_t);
        painter.rect_stroke(
            bar_rect,
            radius,
            Stroke::new(1.5 * scale, border),
            StrokeKind::Outside,
        );

        if shown > 0.0 {
            // Keep the pill shape even at tiny progress values.
            let filled_width = (bar_rect.width() * shown).max(bar_height);
            let filled_rect =
                Rect::from_min_size(bar_rect.min, Vec2::new(filled_width, bar_height));

            let fill = palette::FILL_IDLE.lerp_to_gamma(palette::FILL_HOVER, hover_t);
            painter.rect_filled(filled_rect, radius, fill);

            let highlight = Rect::from_min_size(
                filled_rect.min + Vec2::new(radius * 0.5, scale),
                Vec2::new((filled_rect.width() - radius).max(0.0), bar_height * 0.35),
            );
            painter.rect_filled(
                highlight,
                highlight.height() / 2.0,
                Color32::WHITE.gamma_multiply(0.2),
            );

            draw_position_indicator(
                ui,
                Pos2::new(filled_rect.max.x - radius, bar_rect.center().y),
                scale,
                hover_t,
                dragging,
            );
        }
    }
}

fn draw_position_indicator(ui: &egui::Ui, pos: Pos2, scale: f32, hover_t: f32, dragging: bool) {
    let drag_bonus = if dragging { 1.0 } else { 0.0 };
    let r = (egui::lerp(6.0..=8.5, hover_t) + drag_bonus) * scale;
    let glow = egui::lerp(0.6..=1.4, hover_t);
    let painter = ui.painter();

    painter.circle_filled(
        pos,
        4.0f32.mul_add(scale, r),
        palette::KNOB_GLOW.gamma_multiply(0.14 * glow),
    );
    painter.circle_filled(
        pos,
        2.5f32.mul_add(scale, r),
        palette::KNOB_GLOW.gamma_multiply(0.28 * glow),
    );
    painter.circle_filled(pos, r, palette::KNOB);
    painter.circle_filled(
        pos - Vec2::splat(1.5 * scale),
        r * 0.5,
        Color32::WHITE.gamma_multiply(0.78),
    );
}

const fn sanitize_progress(progress: f32) -> f32 {
    if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
