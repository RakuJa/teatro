use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player::player::{PlayerLayout, palette};
use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::{Color32, Shape, Stroke};
use egui::{Align2, CursorIcon, FontId, Painter, Response, Sense};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Icon {
    Shuffle,
    Speaker,
    SpeakerMuted,
    Solo,
    Stop,
    Play,
    Pause,
    Next,
}

fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| {
        (f32::from(y) - f32::from(x))
            .mul_add(t, f32::from(x))
            .round() as u8
    };
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn arc_points(center: Pos2, radius: f32, from_deg: f32, to_deg: f32, steps: usize) -> Vec<Pos2> {
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let a = (to_deg - from_deg).mul_add(t, from_deg).to_radians();
            Pos2::new(
                radius.mul_add(a.cos(), center.x),
                radius.mul_add(a.sin(), center.y),
            )
        })
        .collect()
}

fn arrow_head(painter: &Painter, tip: Pos2, size: f32, color: Color32) {
    painter.add(Shape::convex_polygon(
        vec![
            tip,
            Pos2::new(tip.x - size, size.mul_add(-0.7, tip.y)),
            Pos2::new(tip.x - size, size.mul_add(0.7, tip.y)),
        ],
        color,
        Stroke::NONE,
    ));
}

fn paint_icon(painter: &Painter, icon: Icon, c: Pos2, s: f32, color: Color32, scale: f32) {
    let stroke = Stroke::new(1.7 * scale, color);
    match icon {
        Icon::Play => {
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(s.mul_add(-0.45, c.x), s.mul_add(-0.75, c.y)),
                    Pos2::new(s.mul_add(-0.45, c.x), s.mul_add(0.75, c.y)),
                    Pos2::new(s.mul_add(0.8, c.x), c.y),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Pause => {
            let w = s * 0.42;
            let h = s * 1.5;
            for dx in [-s * 0.42, s * 0.42] {
                painter.rect_filled(
                    Rect::from_center_size(Pos2::new(c.x + dx, c.y), Vec2::new(w, h)),
                    1.0 * scale,
                    color,
                );
            }
        }
        Icon::Next => {
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(s.mul_add(-0.75, c.x), s.mul_add(-0.65, c.y)),
                    Pos2::new(s.mul_add(-0.75, c.x), s.mul_add(0.65, c.y)),
                    Pos2::new(s.mul_add(0.35, c.x), c.y),
                ],
                color,
                Stroke::NONE,
            ));
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(s.mul_add(0.45, c.x), s.mul_add(-0.65, c.y)),
                    Pos2::new(s.mul_add(0.8, c.x), s.mul_add(0.65, c.y)),
                ),
                0.5 * scale,
                color,
            );
        }
        Icon::Stop => {
            painter.rect_filled(
                Rect::from_center_size(c, Vec2::splat(s * 1.35)),
                1.5 * scale,
                color,
            );
        }
        Icon::Solo => {
            painter.text(
                c,
                Align2::CENTER_CENTER,
                "S",
                FontId::proportional(s * 2.0),
                color,
            );
        }
        Icon::Shuffle => {
            let l = s * 0.95;
            let a = Pos2::new(c.x - l, s.mul_add(-0.55, c.y));
            let b = Pos2::new(c.x - l, s.mul_add(0.55, c.y));
            let a2 = Pos2::new(s.mul_add(-0.3, c.x + l), s.mul_add(0.55, c.y));
            let b2 = Pos2::new(s.mul_add(-0.3, c.x + l), s.mul_add(-0.55, c.y));
            painter.line_segment([a, a2], stroke);
            painter.line_segment([b, b2], stroke);
            arrow_head(
                painter,
                Pos2::new(s.mul_add(0.2, c.x + l), s.mul_add(0.55, c.y)),
                s * 0.6,
                color,
            );
            arrow_head(
                painter,
                Pos2::new(s.mul_add(0.2, c.x + l), s.mul_add(-0.55, c.y)),
                s * 0.6,
                color,
            );
        }
        Icon::Speaker | Icon::SpeakerMuted => {
            let x0 = s.mul_add(-0.9, c.x);
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x0, s.mul_add(-0.35, c.y)),
                    Pos2::new(s.mul_add(0.5, x0), s.mul_add(0.35, c.y)),
                ),
                0.5 * scale,
                color,
            );
            painter.add(Shape::convex_polygon(
                vec![
                    Pos2::new(s.mul_add(0.5, x0), s.mul_add(-0.35, c.y)),
                    Pos2::new(s.mul_add(1.1, x0), s.mul_add(-0.85, c.y)),
                    Pos2::new(s.mul_add(1.1, x0), s.mul_add(0.85, c.y)),
                    Pos2::new(s.mul_add(0.5, x0), s.mul_add(0.35, c.y)),
                ],
                color,
                Stroke::NONE,
            ));
            let wave_center = Pos2::new(s.mul_add(1.1, x0), c.y);
            if icon == Icon::Speaker {
                for r in [s * 0.6, s * 1.05] {
                    painter.add(Shape::line(
                        arc_points(wave_center, r, -42.0, 42.0, 10),
                        stroke,
                    ));
                }
            } else {
                let k = s * 0.35;
                let m = Pos2::new(s.mul_add(1.75, x0), c.y);
                painter.line_segment(
                    [Pos2::new(m.x - k, m.y - k), Pos2::new(m.x + k, m.y + k)],
                    stroke,
                );
                painter.line_segment(
                    [Pos2::new(m.x - k, m.y + k), Pos2::new(m.x + k, m.y - k)],
                    stroke,
                );
            }
        }
    }
}

fn toggle_button(
    ui: &egui::Ui,
    rect: Rect,
    id: &str,
    icon: Icon,
    active: bool,
    accent: Color32,
    tooltip: &str,
    scale: f32,
) -> Response {
    let response = ui
        .interact(rect, ui.id().with(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    let pressed = response.is_pointer_button_down_on();
    let base = if pressed {
        palette::BUTTON_PRESSED
    } else if response.hovered() {
        palette::BUTTON_HOVER
    } else {
        palette::BUTTON_IDLE
    };

    let rounding = 6.0 * scale;
    let painter = ui.painter();

    let icon_color = if active {
        painter.rect_filled(rect, rounding, accent);
        let ring = 1.5 * scale;
        painter.rect_filled(
            rect.shrink(ring),
            (rounding - ring).max(0.0),
            mix(base, accent, 0.22),
        );
        mix(accent, Color32::WHITE, 0.35)
    } else {
        painter.rect_filled(rect, rounding, base);
        if response.hovered() {
            palette::ICON_HOVER
        } else {
            palette::ICON_IDLE
        }
    };

    paint_icon(painter, icon, rect.center(), 6.5 * scale, icon_color, scale);
    response
}

fn round_button(
    ui: &egui::Ui,
    center: Pos2,
    diameter: f32,
    id: &str,
    icon: Icon,
    icon_half_size: f32,
    primary: bool,
    tooltip: &str,
    scale: f32,
) -> Response {
    let rect = Rect::from_center_size(center, Vec2::splat(diameter));
    let response = ui
        .interact(rect, ui.id().with(id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(tooltip);

    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let radius = diameter / 2.0 - if pressed { 1.0 * scale } else { 0.0 };
    let painter = ui.painter();

    let (fill, icon_color) = if primary {
        let fill = if pressed {
            palette::PRIMARY_PRESSED
        } else if hovered {
            palette::PRIMARY_HOVER
        } else {
            palette::PRIMARY
        };
        (fill, palette::ON_PRIMARY)
    } else {
        let fill = if pressed {
            palette::BUTTON_PRESSED
        } else if hovered {
            palette::BUTTON_HOVER
        } else {
            palette::BUTTON_IDLE
        };
        let icon_color = if hovered {
            palette::ICON_HOVER
        } else {
            palette::ICON_IDLE
        };
        (fill, icon_color)
    };

    painter.circle_filled(center, radius, fill);
    paint_icon(painter, icon, center, icon_half_size, icon_color, scale);
    response
}

struct ToggleSpec {
    id: &'static str,
    icon: Icon,
    active: bool,
    accent: Color32,
    tooltip: &'static str,
    flag: PlayerStatus,
    command: CommsCommand,
}

impl AkaiVisualizer {
    /// Flip a status flag, then notify the backend.
    fn toggle_and_send(&self, flag: PlayerStatus, command: CommsCommand) {
        let toggled = self
            .gui_data
            .lock()
            .map(|mut data| data.player_info.status.toggle(flag))
            .is_ok();
        if toggled {
            self.send_command_to_backend(command);
        }
    }

    /// Height the player panel needs so the control rows never overlap.
    pub(crate) fn player_height(content_padding: f32, scale: f32) -> f32 {
        PlayerLayout::new(scale).required_height(content_padding)
    }

    pub(crate) fn draw_control_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let layout = PlayerLayout::new(scale);
        let muted = current_status.is_music_muted();

        let toggles = [
            ToggleSpec {
                id: "shuffle",
                icon: Icon::Shuffle,
                active: current_status.is_shuffle_requested(),
                accent: palette::SHUFFLE,
                tooltip: "Shuffle",
                flag: PlayerStatus::SHUFFLE,
                command: CommsCommand::ShufflePressed {},
            },
            ToggleSpec {
                id: "mute",
                icon: if muted {
                    Icon::SpeakerMuted
                } else {
                    Icon::Speaker
                },
                active: muted,
                accent: palette::MUTE,
                tooltip: if muted { "Unmute" } else { "Mute" },
                flag: PlayerStatus::MUTE_ALL,
                command: CommsCommand::MutePressed {},
            },
            ToggleSpec {
                id: "solo",
                icon: Icon::Solo,
                // Kept from the original: solo is shown when sound is muted.
                active: current_status.is_sound_muted(),
                accent: palette::SOLO,
                tooltip: "Solo music",
                flag: PlayerStatus::SOLO_MUSIC,
                command: CommsCommand::SoloPressed {},
            },
            ToggleSpec {
                id: "stop_all",
                icon: Icon::Stop,
                active: current_status.is_everything_stopped(),
                accent: palette::STOP,
                tooltip: "Stop all",
                flag: PlayerStatus::STOP_ALL,
                command: CommsCommand::StopAllPressed {},
            },
        ];

        for (i, spec) in toggles.iter().enumerate() {
            let rect = layout.toggle_rect(player_rect, content_padding, i);
            let response = toggle_button(
                ui,
                rect,
                spec.id,
                spec.icon,
                spec.active,
                spec.accent,
                spec.tooltip,
                scale,
            );
            if response.clicked() {
                self.toggle_and_send(spec.flag, spec.command);
            }
        }

        self.draw_playlist_toggle(ui, player_rect, content_padding, scale, toggles.len());
    }

    pub(crate) fn draw_playback_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let layout = PlayerLayout::new(scale);
        let cy = layout.transport_center_y(player_rect, content_padding);
        let cx = player_rect.center().x;

        let play_center = Pos2::new(cx, cy);
        let skip_center = Pos2::new(
            cx + layout.play_diameter / 2.0 + layout.transport_gap + layout.skip_diameter / 2.0,
            cy,
        );

        let paused = current_status.is_music_paused();
        let play_response = round_button(
            ui,
            play_center,
            layout.play_diameter,
            "pause_btn",
            if paused { Icon::Play } else { Icon::Pause },
            8.5 * scale,
            true,
            if paused { "Play" } else { "Pause" },
            scale,
        );
        if play_response.clicked() {
            self.toggle_and_send(PlayerStatus::PAUSE_MUSIC, CommsCommand::PausePressed {});
        }

        let skip_response = round_button(
            ui,
            skip_center,
            layout.skip_diameter,
            "skip_btn",
            Icon::Next,
            6.5 * scale,
            false,
            "Next track",
            scale,
        );
        if skip_response.clicked() {
            self.send_command_to_backend(CommsCommand::SkipTrackPressed {});
        }
    }
}
