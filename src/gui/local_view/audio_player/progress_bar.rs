use crate::gui::comms::command::CommsCommand;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;
use egui::{CursorIcon, Id, Sense, Stroke, StrokeKind};

impl AkaiVisualizer {
    /// Draws the progress bar. Returns `Some(position)` (0.0 to 1.0) when the
    /// user clicks or drags to seek.
    pub(crate) fn draw_progress_bar(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        progress: f32,
    ) {
        let progress = if progress.is_finite() {
            progress.clamp(0.0, 1.0)
        } else {
            0.0
        };

        let bar_height = 12.0 * scale;
        let radius = bar_height / 2.0;
        let bar_rect = Rect::from_min_size(
            Pos2::new(
                player_rect.min.x + content_padding,
                player_rect.max.y - content_padding - bar_height,
            ),
            Vec2::new(
                content_padding.mul_add(-2.0, player_rect.width()),
                bar_height,
            ),
        );

        // --- Interaction -------------------------------------------------
        // Hit area is taller than the visual bar so it's easy to grab.
        let hit_rect = bar_rect.expand2(Vec2::new(0.0, 8.0 * scale));
        let id = Id::new("akai_progress_bar");
        let response = ui.interact(hit_rect, id, Sense::click_and_drag());

        let pos_to_progress = |x: f32| ((x - bar_rect.min.x) / bar_rect.width()).clamp(0.0, 1.0);

        let dragging = response.dragged();
        let hovered = response.hovered() || dragging;
        if hovered {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        let drag_id = id.with("drag_target");
        let pending_id = id.with("pending_seek");
        let now = ui.input(|i| i.time);

        // Live position while dragging (display only, nothing sent yet)
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

        // Commit: a plain click, or the end of a drag
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

        // After release, the backend's reported progress lags for a moment, so
        // keep showing the target briefly to avoid the knob snapping back.
        let pending = ui
            .ctx()
            .data(|d| d.get_temp::<(f32, f64)>(pending_id))
            .filter(|(_, t)| now - t < 0.4)
            .map(|(target, _)| target);
        if pending.is_some() {
            ui.ctx().request_repaint();
        }

        let shown = drag_pos.or(pending).unwrap_or(progress);

        // 0.0 -> idle, 1.0 -> hovered/dragging (smoothly animated)
        let hover_t = ui
            .ctx()
            .animate_bool_with_time(id.with("hover"), hovered, 0.12);

        // --- Drawing -----------------------------------------------------
        let painter = ui.painter();

        painter.rect_filled(bar_rect, radius, Color32::from_rgb(35, 35, 45));

        // Hover preview: dim fill from the playhead to the cursor
        if let Some(hover_pos) = response.hover_pos().filter(|_| !dragging) {
            let hover_x = bar_rect
                .width()
                .mul_add(pos_to_progress(hover_pos.x), bar_rect.min.x);
            let played_x = bar_rect.width().mul_add(shown, bar_rect.min.x) * shown;
            if hover_x > played_x {
                let preview = Rect::from_min_max(
                    Pos2::new(played_x, bar_rect.min.y),
                    Pos2::new(hover_x, bar_rect.max.y),
                );
                painter.rect_filled(
                    preview,
                    radius,
                    Color32::from_rgba_unmultiplied(150, 70, 190, 60),
                );
            }
        }

        // Border brightens on hover
        let border = lerp_color(
            Color32::from_rgb(55, 55, 65),
            Color32::from_rgb(95, 80, 115),
            hover_t,
        );
        painter.rect_stroke(
            bar_rect,
            radius,
            Stroke::new(1.5 * scale, border),
            StrokeKind::Outside,
        );

        if shown > 0.0 {
            // Keep the pill shape even at tiny progress values
            let filled_width = (bar_rect.width() * shown).max(bar_height);
            let filled_rect =
                Rect::from_min_size(bar_rect.min, Vec2::new(filled_width, bar_height));

            let fill = lerp_color(
                Color32::from_rgb(150, 70, 190),
                Color32::from_rgb(170, 90, 210),
                hover_t,
            );
            painter.rect_filled(filled_rect, radius, fill);

            let highlight = Rect::from_min_size(
                filled_rect.min + Vec2::new(radius * 0.5, 1.0 * scale),
                Vec2::new(filled_rect.width() - radius, bar_height * 0.35),
            );
            painter.rect_filled(
                highlight,
                highlight.height() / 2.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, 50),
            );

            Self::draw_position_indicator(
                ui,
                filled_rect.max.x - radius,
                bar_rect.center().y,
                scale,
                hover_t,
                dragging,
            );
        }
    }

    fn draw_position_indicator(
        ui: &egui::Ui,
        x: f32,
        y: f32,
        scale: f32,
        hover_t: f32,
        dragging: bool,
    ) {
        // Knob grows on hover, and a bit more while dragging
        let base = 2.5f32.mul_add(hover_t, 6.0) + if dragging { 1.0 } else { 0.0 };
        let r = base * scale;
        let pos = Pos2::new(x, y);
        let painter = ui.painter();

        // Glow gets stronger on hover
        let glow_alpha =
            |a: f32| (a * 0.8f32.mul_add(hover_t, 0.6)).round().clamp(0.0, 255.0) as u8;
        painter.circle_filled(
            pos,
            4.0f32.mul_add(scale, r),
            Color32::from_rgba_unmultiplied(200, 120, 240, glow_alpha(35.)),
        );
        painter.circle_filled(
            pos,
            2.5f32.mul_add(scale, r),
            Color32::from_rgba_unmultiplied(190, 110, 230, glow_alpha(70.)),
        );

        painter.circle_filled(pos, r, Color32::from_rgb(230, 180, 255));
        painter.circle_filled(
            pos - Vec2::splat(1.5 * scale),
            r * 0.5,
            Color32::from_rgba_unmultiplied(255, 255, 255, 200),
        );
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| {
        (f32::from(y) - f32::from(x))
            .mul_add(t, f32::from(x))
            .round() as u8
    };
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}
