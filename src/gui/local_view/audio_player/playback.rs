use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player::button_style::{ButtonStyle, draw_button};
use crate::gui::local_view::audio_player_states::PlayerStatus;
use crate::gui::ui::AkaiVisualizer;
use eframe::emath::{Pos2, Rect, Vec2};
use eframe::epaint::Color32;

impl AkaiVisualizer {
    pub(crate) fn draw_control_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        content_padding: f32,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let button_size = Vec2::new(28.0 * scale, 24.0 * scale);
        let button_spacing = 8.0 * scale;
        let buttons_y = content_padding + player_rect.min.y;
        let buttons_start_x = player_rect.min.x + content_padding;

        let buttons = [
            (
                "shuffle",
                "🔀",
                current_status.is_shuffle_requested(),
                Color32::from_rgb(180, 100, 220),
                PlayerStatus::SHUFFLE,
                CommsCommand::ShufflePressed {},
            ),
            (
                "mute",
                if current_status.is_music_muted() {
                    "🔇"
                } else {
                    "🔊"
                },
                current_status.is_music_muted(),
                Color32::from_rgb(200, 80, 80),
                PlayerStatus::MUTE_ALL,
                CommsCommand::MutePressed {},
            ),
            (
                "solo",
                "S",
                current_status.is_sound_muted(),
                Color32::from_rgb(100, 180, 220),
                PlayerStatus::SOLO_MUSIC,
                CommsCommand::SoloPressed {},
            ),
            (
                "stop_all",
                "⏹",
                current_status.is_everything_stopped(),
                Color32::from_rgb(220, 80, 80),
                PlayerStatus::STOP_ALL,
                CommsCommand::StopAllPressed {},
            ),
        ];

        for (i, (id, icon, is_active, active_color, status_flag, command)) in
            buttons.iter().enumerate()
        {
            let x = (button_size.x + button_spacing).mul_add(i as f32, buttons_start_x);
            let rect = Rect::from_min_size(Pos2::new(x, buttons_y), button_size);

            let style = ButtonStyle::new(scale).active_color(if *is_active {
                *active_color
            } else {
                Color32::from_rgb(60, 60, 70)
            });

            let icon_size = if *id == "solo" { 14.0 } else { 12.0 } * scale;
            let response = draw_button(ui, rect, &style, icon, icon_size, id);

            if response.clicked()
                && matches!(
                    self.gui_data
                        .lock()
                        .map(|mut x| x.player_info.status.toggle(*status_flag)),
                    Ok(())
                )
            {
                self.send_command_to_backend(*command);
            }
        }
        self.draw_playlist_toggle(ui, player_rect, content_padding, scale, buttons.len());
    }

    // Playback buttons (pause, skip)
    pub(crate) fn draw_playback_buttons(
        &self,
        ui: &egui::Ui,
        player_rect: Rect,
        scale: f32,
        current_status: PlayerStatus,
    ) {
        let button_size = Vec2::new(32.0 * scale, 32.0 * scale);
        let bar_height = 12.0 * scale;
        let content_padding = 14.0 * scale;
        let bar_y = player_rect.max.y - content_padding - bar_height;
        let buttons_y = bar_y - button_size.y / 2.0 + bar_height / 2.0;
        let center_x = player_rect.center().x;

        let style = ButtonStyle::new(scale).with_size(button_size);

        // Pause button
        let pause_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(-scale, center_x - button_size.x / 2.0),
                buttons_y + button_size.y / 2.0,
            ),
            button_size,
        );
        let pause_icon = if current_status.is_music_paused() {
            "▶"
        } else {
            "⏸"
        };
        let pause_response = draw_button(
            ui,
            pause_rect,
            &style,
            pause_icon,
            16.0 * scale,
            "pause_btn",
        );

        if pause_response.clicked()
            && matches!(
                self.gui_data
                    .lock()
                    .map(|mut x| x.player_info.status.toggle(PlayerStatus::PAUSE_MUSIC)),
                Ok(())
            )
        {
            self.send_command_to_backend(CommsCommand::PausePressed {});
        }

        // Skip button
        let skip_rect = Rect::from_center_size(
            Pos2::new(
                4.0f32.mul_add(scale, center_x + button_size.x / 2.0),
                buttons_y + button_size.y / 2.0,
            ),
            button_size,
        );
        let skip_response = draw_button(ui, skip_rect, &style, "⏭", 16.0 * scale, "skip_btn");

        if skip_response.clicked() {
            self.send_command_to_backend(CommsCommand::SkipTrackPressed {});
        }
    }
}
