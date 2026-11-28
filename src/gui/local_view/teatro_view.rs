use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::knob::{self, KNOBS};
use crate::gui::theme;
use crate::gui::ui::AkaiVisualizer;
use crate::states::filter_data::knob_filter;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, FontId};
use egui::{Align2, CursorIcon, Frame, Id, Margin, Panel, Sense, Stroke, StrokeKind, Ui};

const CTRL_W: f32 = 880.0;
const CTRL_H: f32 = 484.0;
const PAD: f32 = 20.0;
const SECTION_LABEL_Y: f32 = 14.0;
const SECTION_TOP: f32 = 36.0;
const CARD_GAP: f32 = 12.0;

const PAD_COLS: u8 = 8;
const PAD_ROWS: u8 = 5;
const PAD_SIZE: f32 = 52.0;
const PAD_GAP: f32 = 8.0;

const KNOB_SECTION_GAP: f32 = 36.0;
const KNOB_COLS: usize = 4;
const KNOB_ROW_H: f32 = 146.0;
const KNOB_RADIUS: f32 = 24.0;

const KEYS_LABEL_Y: f32 = 346.0;
const KEYS_TOP: f32 = 366.0;
const WHITE_KEY_H: f32 = 100.0;
const BLACK_KEY_H: f32 = 62.0;
const KEY_COUNT: u8 = 25;
const WHITE_KEY_COUNT: f32 = 15.0;

const TRACKER_DEFAULT_W: f32 = 320.0;
const TRACKER_MIN_W: f32 = 260.0;
const TRACKER_MAX_W: f32 = 520.0;

const IS_WHITE: [bool; 12] = [
    true, false, true, false, true, true, false, true, false, true, false, true,
];

fn pads_width() -> f32 {
    f32::from(PAD_COLS).mul_add(PAD_SIZE, f32::from(PAD_COLS - 1) * PAD_GAP)
}

fn section_label(ui: &Ui, pos: Pos2, title: &str, detail: &str, scale: f32) {
    let painter = ui.painter();
    let title_rect = painter.text(
        pos,
        Align2::LEFT_TOP,
        title,
        FontId::proportional(11.0 * scale),
        theme::TEXT_MUTED,
    );
    if !detail.is_empty() {
        painter.text(
            Pos2::new(8.0f32.mul_add(scale, title_rect.max.x), pos.y),
            Align2::LEFT_TOP,
            detail,
            FontId::proportional(11.0 * scale),
            theme::TEXT_FAINT,
        );
    }
}

impl AkaiVisualizer {
    pub(crate) fn render_visualizer_tab(&mut self, ui: &mut Ui) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::profile_function!();

        Panel::right("information_tracker")
            .resizable(true)
            .default_size(TRACKER_DEFAULT_W)
            .size_range(TRACKER_MIN_W..=TRACKER_MAX_W)
            .show_separator_line(false)
            .frame(Frame::new().inner_margin(Margin {
                left: 12,
                ..Margin::ZERO
            }))
            .show(ui, |ui| self.draw_information_list(ui));

        let area = ui.available_rect_before_wrap();
        let player_base_h = Self::player_height(14.0, 1.0);
        let total_base = Vec2::new(CTRL_W, CTRL_H + CARD_GAP + player_base_h);
        let scale = (area.width() / total_base.x)
            .min(area.height() / total_base.y)
            .clamp(0.5, 1.75);

        let origin = Pos2::new(area.center().x - total_base.x * scale / 2.0, area.min.y);
        let ctrl_rect = Rect::from_min_size(origin, Vec2::new(CTRL_W, CTRL_H) * scale);
        let player_rect = Rect::from_min_size(
            Pos2::new(origin.x, CARD_GAP.mul_add(scale, ctrl_rect.max.y)),
            Vec2::new(CTRL_W, player_base_h) * scale,
        );
        ui.expand_to_include_rect(area);

        self.draw_controller(ui, ctrl_rect, scale);
        self.draw_audio_player(ui, player_rect, scale);
    }

    fn draw_controller(&self, ui: &mut Ui, rect: Rect, scale: f32) {
        ui.painter().rect(
            rect,
            10.0 * scale,
            theme::CONTROLLER_BG,
            Stroke::new(1.0, theme::BORDER),
            StrokeKind::Inside,
        );
        let at = |x: f32, y: f32| rect.min + Vec2::new(x, y) * scale;

        section_label(ui, at(PAD, SECTION_LABEL_Y), "PADS", "music albums", scale);
        self.draw_pads(ui, at(PAD, SECTION_TOP), scale);

        let knobs_x = PAD + pads_width() + KNOB_SECTION_GAP;
        section_label(
            ui,
            at(knobs_x, SECTION_LABEL_Y),
            "KNOBS",
            "drag, scroll or arrow keys (Shift = fine)",
            scale,
        );
        let knobs_w = CTRL_W - knobs_x - PAD;
        self.draw_knobs(
            ui,
            Rect::from_min_size(
                at(knobs_x, SECTION_TOP),
                Vec2::new(knobs_w, 2.0 * KNOB_ROW_H) * scale,
            ),
            scale,
        );

        section_label(
            ui,
            at(PAD, KEYS_LABEL_Y),
            "KEYS",
            "white = sound effects, black = ambience",
            scale,
        );
        self.draw_keyboard(
            ui,
            Rect::from_min_size(
                at(PAD, KEYS_TOP),
                Vec2::new(2.0f32.mul_add(-PAD, CTRL_W), WHITE_KEY_H) * scale,
            ),
            scale,
        );
    }

    fn draw_pads(&self, ui: &Ui, origin: Pos2, scale: f32) {
        let (labels, last_pad_pressed) = self.gui_data.lock().map_or_else(
            |_| (Vec::new(), None),
            |g| (g.data.pad_labels.clone(), g.data.last_pad_pressed),
        );
        let mut clicked_pad = None;

        for row in 0..PAD_ROWS {
            for col in 0..PAD_COLS {
                // Bottom-left pad is 0, as on the hardware.
                let idx = (PAD_ROWS - 1 - row) * PAD_COLS + col;
                let min = origin
                    + Vec2::new(
                        f32::from(col) * (PAD_SIZE + PAD_GAP),
                        f32::from(row) * (PAD_SIZE + PAD_GAP),
                    ) * scale;
                let pad_rect = Rect::from_min_size(min, Vec2::splat(PAD_SIZE * scale));
                let label = labels
                    .get(usize::from(idx))
                    .map(String::as_str)
                    .filter(|l| !l.is_empty());
                if draw_pad(
                    ui,
                    pad_rect,
                    idx,
                    label,
                    last_pad_pressed == Some(idx),
                    scale,
                ) {
                    clicked_pad = Some(idx);
                }
            }
        }

        if let Some(key) = clicked_pad {
            self.send_command_to_backend(CommsCommand::PadPressed { key });
        }
    }

    fn draw_knobs(&self, ui: &mut Ui, area: Rect, scale: f32) {
        let values: [f32; 8] = self.gui_data.lock().map_or([0.0; 8], |g| {
            std::array::from_fn(|i| {
                g.data
                    .knob_values
                    .get(&KNOBS[i].index)
                    .copied()
                    .unwrap_or(0.0)
            })
        });
        let cell_w = area.width() / KNOB_COLS as f32;
        let radius = KNOB_RADIUS * scale;

        for (i, spec) in KNOBS.iter().enumerate() {
            let cell = Rect::from_min_size(
                area.min
                    + Vec2::new(
                        (i % KNOB_COLS) as f32 * cell_w,
                        (i / KNOB_COLS) as f32 * KNOB_ROW_H * scale,
                    ),
                Vec2::new(cell_w, KNOB_ROW_H * scale),
            );
            let color = spec.channel.color();
            let id = Id::new(("teatro_knob", spec.index));
            let center = Pos2::new(cell.center().x, 10.0f32.mul_add(scale, cell.min.y + radius));

            let (response, mut ticks) = knob::knob(ui, id, center, radius, values[i], color, scale);
            let filter = knob_filter(spec.index);
            let current = filter.map_or_else(
                || knob::format_percent(values[i]),
                |_| {
                    format!(
                        "{} ({})",
                        knob::format_knob_value(values[i], filter),
                        knob::format_percent(values[i])
                    )
                },
            );
            response.on_hover_text(format!(
                "K{} - {} {}: {current}\nDrag or scroll to turn, Shift for fine steps\n\
                 Click to focus, then use the arrow keys",
                spec.index,
                spec.channel.label(),
                spec.function
            ));

            let painter = ui.painter();
            let text_y = 16.0f32.mul_add(scale, center.y + radius);
            painter.text(
                Pos2::new(center.x, text_y),
                Align2::CENTER_TOP,
                spec.channel.label(),
                FontId::proportional(10.0 * scale),
                color,
            );
            painter.text(
                Pos2::new(center.x, 14.0f32.mul_add(scale, text_y)),
                Align2::CENTER_TOP,
                spec.function,
                FontId::proportional(12.5 * scale),
                theme::TEXT_HOVER,
            );

            let value_rect = Rect::from_center_size(
                Pos2::new(center.x, 44.0f32.mul_add(scale, text_y)),
                Vec2::new(64.0, 22.0) * scale,
            );
            ticks += knob::value_field(ui, id, value_rect, values[i], filter, scale);

            if ticks != 0 {
                self.send_command_to_backend(CommsCommand::KnobTurned {
                    knob: spec.index,
                    steps: ticks,
                });
            }
        }
    }

    fn draw_keyboard(&self, ui: &Ui, area: Rect, scale: f32) {
        let white_w = area.width() / WHITE_KEY_COUNT;
        let black_w = white_w * 0.6;
        let mut pressed = None;

        // White keys first so black keys paint (and hit-test) on top.
        let mut white_idx = 0u8;
        for key in 0..KEY_COUNT {
            if !IS_WHITE[usize::from(key % 12)] {
                continue;
            }
            let rect = Rect::from_min_size(
                area.min + Vec2::new(f32::from(white_idx) * white_w, 0.0),
                Vec2::new(white_w, WHITE_KEY_H * scale),
            )
            .shrink2(Vec2::new(1.0 * scale, 0.0));
            white_idx += 1;
            let tooltip = format!("Sound effect {white_idx}");
            if draw_key(ui, rect, true, &white_idx.to_string(), &tooltip, scale) {
                pressed = Some(CommsCommand::WhiteKeyPressed { key: key + 1 });
            }
        }

        let mut white_seen = 0u8;
        let mut black_idx = 0u8;
        for key in 0..KEY_COUNT {
            if IS_WHITE[usize::from(key % 12)] {
                white_seen += 1;
                continue;
            }
            black_idx += 1;
            let x = f32::from(white_seen).mul_add(white_w, area.min.x) - black_w / 2.0;
            let rect = Rect::from_min_size(
                Pos2::new(x, area.min.y),
                Vec2::new(black_w, BLACK_KEY_H * scale),
            );
            let tooltip = format!("Ambience {black_idx}");
            if draw_key(ui, rect, false, "", &tooltip, scale) {
                pressed = Some(CommsCommand::BlackKeyPressed { key: key + 1 });
            }
        }

        if let Some(command) = pressed {
            self.send_command_to_backend(command);
        }
    }
}

/// Paints one pad and returns whether it was clicked.
fn draw_pad(ui: &Ui, rect: Rect, idx: u8, label: Option<&str>, active: bool, scale: f32) -> bool {
    let response = ui
        .interact(rect, Id::new(("teatro_pad", idx)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(label.map_or_else(
            || format!("Pad {idx}: empty (albums fill the pads in folder-name order)"),
            |l| format!("Pad {idx}: {l}"),
        ));
    let hover_t =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.08);
    let pressed = response.is_pointer_button_down_on();

    let base = if active {
        theme::ACCENT
    } else if label.is_some() {
        theme::PAD_ALBUM
    } else {
        theme::PAD_EMPTY
    };
    let fill = if pressed {
        base.lerp_to_gamma(Color32::BLACK, 0.2)
    } else {
        base.lerp_to_gamma(Color32::WHITE, 0.12 * hover_t)
    };

    let radius = 6.0 * scale;
    let painter = ui.painter();
    if active {
        painter.rect_filled(
            rect.expand(4.0 * scale),
            3.0f32.mul_add(scale, radius),
            theme::ACCENT.gamma_multiply(0.25),
        );
    }
    painter.rect_filled(rect, radius, fill);
    painter.rect_filled(
        Rect::from_min_size(rect.min, Vec2::new(rect.width(), rect.height() * 0.45)),
        egui::epaint::CornerRadiusF32 {
            nw: radius,
            ne: radius,
            sw: 0.0,
            se: 0.0,
        },
        Color32::from_white_alpha(10),
    );

    let number_color = if label.is_some() || active {
        Color32::from_white_alpha(170)
    } else {
        theme::TEXT_FAINT
    };
    painter.text(
        rect.min + Vec2::splat(5.0 * scale),
        Align2::LEFT_TOP,
        idx.to_string(),
        FontId::proportional(10.0 * scale),
        number_color,
    );

    if let Some(label) = label {
        let galley = painter.layout(
            label.to_owned(),
            FontId::proportional(11.5 * scale),
            Color32::WHITE,
            8.0f32.mul_add(-scale, rect.width()),
        );
        let pos = Pos2::new(
            rect.center().x - galley.size().x / 2.0,
            6.0f32.mul_add(scale, rect.center().y - galley.size().y / 2.0),
        );
        painter
            .with_clip_rect(rect.shrink(2.0 * scale))
            .galley(pos, galley, Color32::WHITE);
    }
    response.clicked()
}

/// Paints one piano key and returns whether it was clicked.
fn draw_key(ui: &Ui, rect: Rect, white: bool, label: &str, tooltip: &str, scale: f32) -> bool {
    let response = ui
        .interact(rect, Id::new(("teatro_key", tooltip)), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);
    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();

    let (fill, accent) = if white {
        let fill = if pressed {
            Color32::from_rgb(205, 205, 212)
        } else if hovered {
            Color32::from_rgb(255, 255, 255)
        } else {
            Color32::from_rgb(232, 232, 238)
        };
        (fill, theme::EFFECTS)
    } else {
        let fill = if pressed {
            Color32::from_rgb(16, 16, 20)
        } else if hovered {
            Color32::from_rgb(52, 52, 62)
        } else {
            Color32::from_rgb(28, 28, 34)
        };
        (fill, theme::AMBIENCE)
    };

    let radius = egui::epaint::CornerRadiusF32 {
        nw: 0.0,
        ne: 0.0,
        sw: 4.0 * scale,
        se: 4.0 * scale,
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        radius,
        fill,
        Stroke::new(1.0 * scale, Color32::from_rgb(20, 20, 24)),
        StrokeKind::Inside,
    );
    if hovered || pressed {
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(
                    3.0f32.mul_add(scale, rect.min.x),
                    6.0f32.mul_add(-scale, rect.max.y),
                ),
                Pos2::new(
                    3.0f32.mul_add(-scale, rect.max.x),
                    3.0f32.mul_add(-scale, rect.max.y),
                ),
            ),
            1.5 * scale,
            accent,
        );
    }
    if !label.is_empty() {
        painter.text(
            Pos2::new(rect.center().x, 12.0f32.mul_add(-scale, rect.max.y)),
            Align2::CENTER_BOTTOM,
            label,
            FontId::proportional(10.0 * scale),
            Color32::from_rgb(130, 130, 140),
        );
    }
    response.clicked()
}
