use crate::audio::output::OutputKind;
use crate::gui::comms::command::CommsCommand;
use crate::gui::icons::{self, Icon};
use crate::gui::theme;
use crate::gui::ui::AkaiVisualizer;
use crate::states::settings_data::SettingsData;
use egui::{
    Align, Button, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Frame, Key,
    KeyboardShortcut, Layout, Margin, Modifiers, Rect, Response, RichText, ScrollArea, Sense,
    Stroke, StrokeKind, TextEdit, Ui, Vec2,
};
use rfd::FileDialog;
use std::env;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

const CONTENT_MAX_WIDTH: f32 = 760.0;
const ACTION_BAR_HEIGHT: f32 = 56.0;
const INPUT_HEIGHT: f32 = 28.0;
const SAVED_NOTICE_SECS: f64 = 2.5;
const ECO_HZ: f32 = 30.0;
const SMOOTH_HZ: f32 = 60.0;
const MIN_HZ: f32 = 5.0;
const SNOWFLAKE_LEN: std::ops::RangeInclusive<usize> = 17..=20;
const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

enum SaveFeedback {
    Saved { at: f64 },
    Failed(String),
}

pub struct SettingsViewState {
    saved: SettingsData,
    feedback: Option<SaveFeedback>,
}

impl SettingsViewState {
    pub const fn new(saved: SettingsData) -> Self {
        Self {
            saved,
            feedback: None,
        }
    }
}

enum Validation {
    Ok(&'static str),
    Warn(&'static str),
}

fn music_dir() -> anyhow::Result<PathBuf> {
    let output = std::process::Command::new("xdg-user-dir")
        .arg("MUSIC")
        .output()?;

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    Ok(
        if path.is_empty()
            && let Some(home) = dirs::home_dir()
        {
            home.join("Music")
        } else {
            PathBuf::from(path)
        },
    )
}

fn folder_dialog(current: &str) -> FileDialog {
    let current = Path::new(current);
    if current.is_dir() {
        return FileDialog::new().set_directory(current);
    }
    music_dir().map_or_else(
        |_| FileDialog::new(),
        |path| FileDialog::new().set_directory(path),
    )
}

fn validate_folder(path: &str) -> Validation {
    if path.trim().is_empty() {
        Validation::Warn("No folder selected")
    } else if Path::new(path).is_dir() {
        Validation::Ok("Folder found")
    } else {
        Validation::Warn("Folder not found: tracks in it won't load")
    }
}

fn validate_snowflake(id: &str) -> Validation {
    if id.is_empty() {
        Validation::Warn("Required for Discord output")
    } else if SNOWFLAKE_LEN.contains(&id.len()) {
        Validation::Ok("Looks valid")
    } else {
        Validation::Warn("Discord IDs are usually 17-20 digits long")
    }
}

fn validate_token(token: &str) -> Validation {
    if token.trim().is_empty() {
        Validation::Warn("Required for Discord output")
    } else {
        Validation::Ok("Token set")
    }
}

fn validation_line(ui: &mut Ui, validation: &Validation) {
    let (icon, text, color) = match validation {
        Validation::Ok(t) => (Icon::Check, *t, theme::SUCCESS),
        Validation::Warn(t) => (Icon::Warning, *t, theme::WARNING),
    };
    ui.add_space(2.0);
    icons::icon_label(ui, icon, text, 11.5, color);
}

fn centered_column<R>(ui: &mut Ui, max_width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    let width = ui.available_width().min(max_width);
    let side = (ui.available_width() - width) / 2.0;
    ui.horizontal(|ui| {
        ui.add_space(side);
        ui.vertical(|ui| {
            ui.set_width(width);
            add(ui)
        })
        .inner
    })
    .inner
}

fn section(ui: &mut Ui, title: &str, subtitle: &str, add: impl FnOnce(&mut Ui)) {
    Frame::new()
        .fill(theme::CARD_BG)
        .stroke(Stroke::new(1.0, theme::BORDER))
        .corner_radius(10)
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).size(16.0).strong().color(theme::TEXT));
            ui.add_space(2.0);
            ui.label(RichText::new(subtitle).size(12.5).color(theme::TEXT_MUTED));
            ui.add_space(14.0);
            add(ui);
        });
    ui.add_space(14.0);
}

fn field_label(ui: &mut Ui, label: &str, hint: &str) {
    ui.label(RichText::new(label).size(13.5).color(theme::TEXT));
    if !hint.is_empty() {
        ui.label(RichText::new(hint).size(11.5).color(theme::TEXT_FAINT));
    }
    ui.add_space(4.0);
}

fn input_row<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    let height = ui.spacing().interact_size.y.max(INPUT_HEIGHT);
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), height),
        Layout::right_to_left(Align::Center),
        add,
    )
    .inner
}

fn folder_field(ui: &mut Ui, label: &str, hint: &str, value: &mut String) {
    field_label(ui, label, hint);
    input_row(ui, |ui| {
        if ui
            .add(Button::new("Browse...").min_size(Vec2::new(0.0, INPUT_HEIGHT)))
            .on_hover_text("Pick a folder")
            .clicked()
            && let Some(path) = folder_dialog(value).pick_folder()
        {
            *value = path.display().to_string();
        }
        ui.add(
            TextEdit::singleline(value)
                .hint_text("/path/to/folder")
                .desired_width(f32::INFINITY)
                .background_color(theme::INPUT_BG)
                .margin(Margin::symmetric(8, 6)),
        );
    });
    validation_line(ui, &validate_folder(value));
    ui.add_space(12.0);
}

fn snowflake_field(ui: &mut Ui, label: &str, hint: &str, value: &mut String) {
    field_label(ui, label, hint);
    let response = ui.add(
        TextEdit::singleline(value)
            .hint_text("e.g. 123456789012345678")
            .desired_width(f32::INFINITY)
            .background_color(theme::INPUT_BG)
            .margin(Margin::symmetric(8, 6)),
    );
    if response.changed() {
        value.retain(|c| c.is_ascii_digit());
    }
    validation_line(ui, &validate_snowflake(value));
    ui.add_space(12.0);
}

const fn output_title(kind: OutputKind) -> &'static str {
    match kind {
        OutputKind::Local => "PC audio",
        OutputKind::Discord => "Discord",
    }
}

const fn output_icon(kind: OutputKind) -> Icon {
    match kind {
        OutputKind::Local => Icon::Speaker,
        OutputKind::Discord => Icon::Headphones,
    }
}

const CARD_ICON_SIZE: f32 = 16.0;
const CARD_ICON_GAP: f32 = 8.0;

const fn output_description(kind: OutputKind) -> &'static str {
    match kind {
        OutputKind::Local => "Play through this computer's speakers or headphones.",
        OutputKind::Discord => "Stream everything into a voice channel through your bot.",
    }
}

fn choice_card(ui: &mut Ui, width: f32, kind: OutputKind, selected: bool) -> Response {
    let pad = Vec2::new(14.0, 12.0);
    let radio = 7.0;
    let text_width = 3.0f32
        .mul_add(-radio, 2.0f32.mul_add(-pad.x, width))
        .max(10.0);

    let title = ui.painter().layout_no_wrap(
        output_title(kind).to_owned(),
        FontId::proportional(14.5),
        Color32::PLACEHOLDER,
    );
    let desc = ui.painter().layout(
        output_description(kind).to_owned(),
        FontId::proportional(12.0),
        Color32::PLACEHOLDER,
        text_width,
    );
    let height = pad.y.mul_add(2.0, title.size().y + 4.0 + desc.size().y);
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::RadioButton,
            true,
            selected,
            output_title(kind),
        )
    });

    let hover_t =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    let (fill, border) = if selected {
        (
            theme::CARD_BG.lerp_to_gamma(theme::ACCENT, 0.14),
            theme::ACCENT,
        )
    } else {
        (
            theme::CARD_BG.lerp_to_gamma(Color32::WHITE, 0.03 * hover_t),
            theme::BORDER.lerp_to_gamma(theme::BORDER_STRONG, hover_t),
        )
    };
    painter.rect(
        rect,
        CornerRadius::same(8),
        fill,
        Stroke::new(if selected { 1.5 } else { 1.0 }, border),
        StrokeKind::Inside,
    );

    let radio_center = egui::pos2(rect.max.x - pad.x - radio, rect.min.y + pad.y + radio);
    painter.circle_stroke(
        radio_center,
        radio,
        Stroke::new(
            1.5,
            if selected {
                theme::ACCENT
            } else {
                theme::BORDER_STRONG
            },
        ),
    );
    if selected {
        painter.circle_filled(radio_center, radio * 0.5, theme::ACCENT);
    }

    let text_origin = rect.min + pad;
    let title_color = if selected {
        theme::TEXT
    } else {
        theme::TEXT_MUTED.lerp_to_gamma(theme::TEXT_HOVER, hover_t)
    };
    let title_height = title.size().y;
    icons::paint(
        painter,
        output_icon(kind),
        Rect::from_center_size(
            text_origin + Vec2::new(CARD_ICON_SIZE / 2.0, title_height / 2.0),
            Vec2::splat(CARD_ICON_SIZE),
        ),
        title_color,
    );
    painter.galley(
        text_origin + Vec2::new(CARD_ICON_SIZE + CARD_ICON_GAP, 0.0),
        title,
        title_color,
    );
    painter.galley(
        text_origin + Vec2::new(0.0, title_height + 4.0),
        desc,
        theme::TEXT_FAINT,
    );
    response
}

fn preset_button(ui: &mut Ui, label: &str, hz: f32, current: &mut f32) {
    let selected = (*current - hz).abs() < 0.5;
    if ui.selectable_label(selected, label).clicked() {
        *current = hz;
    }
}

impl AkaiVisualizer {
    fn save_settings(&mut self, now: f64) {
        debug!("Saving settings data");
        let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yml".to_string());
        if let Err(e) = self.settings_data.write_to_config(&config_path) {
            warn!("Failed to save settings {e:?}");
            self.settings_view.feedback = Some(SaveFeedback::Failed(e.to_string()));
            return;
        }

        if let Ok(g_d) = self.gui_data.lock()
            && let Ok(mut s) = g_d.data.settings_data.lock()
        {
            s.copy_data(&self.settings_data);
        }
        self.settings_data.apply_audio();
        self.send_command_to_watchdog(CommsCommand::Refresh {});

        self.settings_view.saved = self.settings_data.clone();
        self.settings_view.feedback = Some(SaveFeedback::Saved { at: now });
    }

    fn revert_settings(&mut self) {
        self.settings_data = self.settings_view.saved.clone();
        self.settings_view.feedback = None;
    }

    pub fn render_settings_tab(&mut self, ui: &mut Ui) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::profile_function!();
        let selection = &mut ui.visuals_mut().selection;
        selection.bg_fill = theme::ACCENT_DIM;
        selection.stroke = Stroke::new(1.0, theme::TEXT);

        let content_height = (ui.available_height() - ACTION_BAR_HEIGHT).max(0.0);
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(content_height)
            .show(ui, |ui| {
                centered_column(ui, CONTENT_MAX_WIDTH, |ui| {
                    ui.add_space(12.0);
                    Self::page_header(ui);
                    self.library_section(ui);
                    self.audio_output_section(ui);
                    self.performance_section(ui);
                });
            });

        // Drawn after the form so it reflects edits made this frame.
        let dirty = self.settings_data != self.settings_view.saved;
        let now = ui.input(|i| i.time);
        if dirty && ui.ctx().input_mut(|i| i.consume_shortcut(&SAVE_SHORTCUT)) {
            self.save_settings(now);
        }
        self.action_bar(ui, now);
    }

    fn page_header(ui: &mut Ui) {
        ui.label(
            RichText::new("Settings")
                .font(FontId::new(24.0, FontFamily::Name("Pixelify".into())))
                .color(theme::ACCENT),
        );
        ui.label(
            RichText::new("Changes take effect when you save them.")
                .size(12.5)
                .color(theme::TEXT_MUTED),
        );
        ui.add_space(16.0);
    }

    fn library_section(&mut self, ui: &mut Ui) {
        let s = &mut self.settings_data;
        section(
            ui,
            "Sound library",
            "Folders Teatro loads audio from.",
            |ui| {
                folder_field(
                    ui,
                    "Music",
                    "One subfolder per pad, in name order (e.g. 00_Tavern, 01_Town).",
                    &mut s.music_folder,
                );
                folder_field(
                    ui,
                    "Ambience",
                    "Audio files triggered by the black keys.",
                    &mut s.ambience_folder,
                );
                folder_field(
                    ui,
                    "Sound effects",
                    "Audio files triggered by the white keys.",
                    &mut s.sound_effect_folder,
                );
            },
        );
    }

    fn audio_output_section(&mut self, ui: &mut Ui) {
        let s = &mut self.settings_data;
        section(
            ui,
            "Audio output",
            "Choose where music, ambience and effects are played.",
            |ui| {
                let gap = 10.0;
                let card_width = (ui.available_width() - gap) / 2.0;
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for kind in [OutputKind::Local, OutputKind::Discord] {
                        if choice_card(ui, card_width, kind, s.audio_output == kind).clicked() {
                            s.audio_output = kind;
                        }
                    }
                });

                if s.audio_output == OutputKind::Discord {
                    ui.add_space(16.0);
                    Self::discord_fields(ui, s);
                }
            },
        );
    }

    fn discord_fields(ui: &mut Ui, s: &mut SettingsData) {
        field_label(
            ui,
            "Bot token",
            "From the Discord Developer Portal > your application > Bot.",
        );
        let show_id = ui.id().with("show_discord_token");
        let mut show = ui.data(|d| d.get_temp::<bool>(show_id).unwrap_or(false));
        input_row(ui, |ui| {
            if ui
                .add(
                    Button::new(if show { "Hide" } else { "Show" })
                        .min_size(Vec2::new(56.0, INPUT_HEIGHT)),
                )
                .on_hover_text("Toggle token visibility")
                .clicked()
            {
                show = !show;
            }
            ui.add(
                TextEdit::singleline(&mut s.discord_bot_token)
                    .password(!show)
                    .hint_text("Paste your bot token")
                    .desired_width(f32::INFINITY)
                    .background_color(theme::INPUT_BG)
                    .margin(Margin::symmetric(8, 6)),
            );
        });
        ui.data_mut(|d| d.insert_temp(show_id, show));
        validation_line(ui, &validate_token(&s.discord_bot_token));
        ui.add_space(12.0);

        snowflake_field(
            ui,
            "Server ID",
            "The server (guild) the bot should join.",
            &mut s.discord_server_id,
        );
        snowflake_field(
            ui,
            "Voice channel ID",
            "The voice channel the bot streams into.",
            &mut s.discord_voice_channel_id,
        );

        icons::icon_label(
            ui,
            Icon::Info,
            "To copy IDs, enable Developer Mode in Discord (Settings > Advanced), \
             then right-click the server or channel > Copy ID.",
            11.5,
            theme::TEXT_MUTED,
        );
    }

    fn performance_section(&mut self, ui: &mut Ui) {
        let s = &mut self.settings_data;
        section(
            ui,
            "Performance",
            "How often the player redraws while music is playing. \
             When idle, Teatro only redraws on input.",
            |ui| {
                let detected = s.detected_display_hz.round().max(1.0);
                field_label(
                    ui,
                    "Refresh rate",
                    &format!("Your display runs at {detected:.0} Hz. Lower values use less CPU."),
                );

                let max_hz = detected.max(s.repaint_display_hz).max(SMOOTH_HZ);
                ui.spacing_mut().slider_width = (ui.available_width() - 90.0).max(100.0);
                ui.add(
                    egui::Slider::new(&mut s.repaint_display_hz, MIN_HZ..=max_hz)
                        .suffix(" Hz")
                        .trailing_fill(true)
                        .fixed_decimals(0)
                        .step_by(1.0)
                        .clamping(egui::SliderClamping::Edits),
                );

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Presets").size(12.0).color(theme::TEXT_FAINT));
                    preset_button(ui, "Eco (30 Hz)", ECO_HZ, &mut s.repaint_display_hz);
                    preset_button(ui, "Smooth (60 Hz)", SMOOTH_HZ, &mut s.repaint_display_hz);
                    if detected > SMOOTH_HZ {
                        preset_button(
                            ui,
                            &format!("Display ({detected:.0} Hz)"),
                            detected,
                            &mut s.repaint_display_hz,
                        );
                    }
                });
            },
        );
    }

    fn action_bar(&mut self, ui: &mut Ui, now: f64) {
        let dirty = self.settings_data != self.settings_view.saved;
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), ACTION_BAR_HEIGHT),
            Sense::hover(),
        );
        ui.painter().hline(
            rect.x_range(),
            rect.min.y + 0.5,
            Stroke::new(1.0, theme::BORDER),
        );

        let width = rect.width().min(CONTENT_MAX_WIDTH);
        let bar = Rect::from_center_size(rect.center(), Vec2::new(width, rect.height()));
        let mut ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bar)
                .layout(Layout::right_to_left(Align::Center)),
        );

        let save = ui
            .add_enabled(
                dirty,
                Button::new(RichText::new("Save changes").strong().color(if dirty {
                    theme::ON_ACCENT
                } else {
                    theme::TEXT_FAINT
                }))
                .fill(if dirty { theme::ACCENT } else { theme::CARD_BG })
                .corner_radius(6)
                .min_size(Vec2::new(140.0, 32.0)),
            )
            .on_hover_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT));
        if save.clicked() {
            self.save_settings(now);
        }

        let revert = ui
            .add_enabled(
                dirty,
                Button::new("Revert")
                    .corner_radius(6)
                    .min_size(Vec2::new(90.0, 32.0)),
            )
            .on_hover_text("Discard unsaved changes");
        if revert.clicked() {
            self.revert_settings();
        }

        ui.add_space(8.0);
        self.status_label(&mut ui, dirty, now);
    }

    fn status_label(&self, ui: &mut Ui, dirty: bool, now: f64) {
        enum Marker {
            Icon(Icon),
            Dot,
            None,
        }
        let (marker, text, color) = match &self.settings_view.feedback {
            Some(SaveFeedback::Failed(e)) => (
                Marker::Icon(Icon::Warning),
                format!("Couldn't save: {e}"),
                theme::ERROR,
            ),
            _ if dirty => (Marker::Dot, "Unsaved changes".to_owned(), theme::WARNING),
            Some(SaveFeedback::Saved { at }) if now - at < SAVED_NOTICE_SECS => {
                // Repaint once more so the notice disappears without user input.
                ui.ctx()
                    .request_repaint_after_secs((SAVED_NOTICE_SECS - (now - at)) as f32);
                (
                    Marker::Icon(Icon::Check),
                    "Settings saved".to_owned(),
                    theme::SUCCESS,
                )
            }
            _ => (
                Marker::None,
                "All changes saved".to_owned(),
                theme::TEXT_FAINT,
            ),
        };
        ui.add(egui::Label::new(RichText::new(text).size(12.5).color(color)).truncate());
        match marker {
            Marker::Icon(icon) => {
                icons::icon(ui, icon, 13.0, color);
            }
            Marker::Dot => {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                ui.painter().circle_filled(rect.center(), 3.5, color);
            }
            Marker::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snowflake_validation() {
        assert!(matches!(validate_snowflake(""), Validation::Warn(_)));
        assert!(matches!(validate_snowflake("123"), Validation::Warn(_)));
        assert!(matches!(
            validate_snowflake("123456789012345678"),
            Validation::Ok(_)
        ));
    }

    #[test]
    fn folder_validation() {
        assert!(matches!(validate_folder("  "), Validation::Warn(_)));
        assert!(matches!(
            validate_folder("/definitely/not/a/real/folder"),
            Validation::Warn(_)
        ));
        let tmp = env::temp_dir();
        assert!(matches!(
            validate_folder(&tmp.display().to_string()),
            Validation::Ok(_)
        ));
    }
}
