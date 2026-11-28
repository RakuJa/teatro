use crate::gui::comms::command::CommsCommand;
use crate::gui::icons::{self, Icon};
use crate::gui::local_view::audio_player::player::{PlayerLayout, palette};
use crate::gui::local_view::audio_player::playlist_data::draw_playlist_toggle;
use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;
use egui::{CursorIcon, Painter, Response, Sense};

/// Player icons are designed around a center and a "half size"; their
/// strokes reach about 1.2 half sizes out, so that is the square they get.
const ICON_EXTENT: f32 = 2.4;

fn paint_icon(painter: &Painter, icon: Icon, center: Pos2, half_size: f32, color: Color32) {
    icons::paint(
        painter,
        icon,
        Rect::from_center_size(center, Vec2::splat(half_size * ICON_EXTENT)),
        color,
    );
}

fn toggle_button(ui: &egui::Ui, rect: Rect, spec: &ToggleSpec, scale: f32) -> Response {
    let response = ui
        .interact(rect, ui.id().with(spec.id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(spec.tooltip);

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

    let icon_color = if spec.active {
        painter.rect_filled(rect, rounding, spec.accent);
        let ring = 1.5 * scale;
        painter.rect_filled(
            rect.shrink(ring),
            (rounding - ring).max(0.0),
            base.lerp_to_gamma(spec.accent, 0.22),
        );
        spec.accent.lerp_to_gamma(Color32::WHITE, 0.35)
    } else {
        painter.rect_filled(rect, rounding, base);
        if response.hovered() {
            palette::ICON_HOVER
        } else {
            palette::ICON_IDLE
        }
    };

    paint_icon(painter, spec.icon, rect.center(), 6.5 * scale, icon_color);
    response
}

struct RoundSpec {
    id: &'static str,
    icon: Icon,
    icon_half_size: f32,
    primary: bool,
    tooltip: &'static str,
}

fn round_button(
    ui: &egui::Ui,
    center: Pos2,
    diameter: f32,
    spec: &RoundSpec,
    scale: f32,
) -> Response {
    let rect = Rect::from_center_size(center, Vec2::splat(diameter));
    let response = ui
        .interact(rect, ui.id().with(spec.id), Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(spec.tooltip);

    let pressed = response.is_pointer_button_down_on();
    let hovered = response.hovered();
    let radius = diameter / 2.0 - if pressed { 1.0 * scale } else { 0.0 };
    let painter = ui.painter();

    let (fill, icon_color) = if spec.primary {
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
    paint_icon(painter, spec.icon, center, spec.icon_half_size, icon_color);
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
            if toggle_button(ui, rect, spec, scale).clicked() {
                self.toggle_and_send(spec.flag, spec.command);
            }
        }

        draw_playlist_toggle(
            ui,
            layout.toggle_rect(player_rect, content_padding, toggles.len()),
            scale,
        );
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
            &RoundSpec {
                id: "pause_btn",
                icon: if paused { Icon::Play } else { Icon::Pause },
                icon_half_size: 8.5 * scale,
                primary: true,
                tooltip: if paused { "Play" } else { "Pause" },
            },
            scale,
        );
        if play_response.clicked() {
            self.toggle_and_send(PlayerStatus::PAUSE_MUSIC, CommsCommand::PausePressed {});
        }

        let skip_response = round_button(
            ui,
            skip_center,
            layout.skip_diameter,
            &RoundSpec {
                id: "skip_btn",
                icon: Icon::Next,
                icon_half_size: 6.5 * scale,
                primary: false,
                tooltip: "Next track",
            },
            scale,
        );
        if skip_response.clicked() {
            self.send_command_to_backend(CommsCommand::SkipTrackPressed {});
        }
    }
}
