use crate::audio::output::OutputKind;
use crate::gui::icons::{self, Icon};
use crate::gui::local_view::audio_player::utils::title_from_path;
use crate::gui::theme;
use crate::gui::ui::AkaiVisualizer;
use egui::text::{LayoutJob, TextWrapping};
use egui::{
    Align, Color32, CursorIcon, FontFamily, FontId, Frame, Id, Key, KeyboardShortcut, Layout,
    Margin, Modifiers, Panel, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType,
};

const HEADER_HEIGHT: f32 = 46.0;
const STATUS_HEIGHT: f32 = 24.0;
const SIDE_MARGIN: i8 = 14;
const TAB_PAD_X: f32 = 14.0;
const TAB_FONT_SIZE: f32 = 14.0;
const TAB_ICON_SIZE: f32 = 15.0;
const TAB_ICON_GAP: f32 = 7.0;
const INDICATOR_HEIGHT: f32 = 3.0;
const INDICATOR_INSET: f32 = 10.0;
const INDICATOR_ANIM_SECS: f32 = 0.18;
const HOVER_ANIM_SECS: f32 = 0.1;
const NOW_PLAYING_MAX_WIDTH: f32 = 280.0;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CurrentTab {
    Visualizer,
    #[cfg(feature = "bybe")]
    WebView,
    Settings,
    LogViewer,
}

impl CurrentTab {
    pub const ALL: &'static [Self] = &[
        Self::Visualizer,
        #[cfg(feature = "bybe")]
        Self::WebView,
        Self::Settings,
        Self::LogViewer,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Visualizer => "Core",
            #[cfg(feature = "bybe")]
            Self::WebView => "BYBE",
            Self::Settings => "Settings",
            Self::LogViewer => "Logs",
        }
    }

    const fn icon(self) -> Icon {
        match self {
            Self::Visualizer => Icon::Keys,
            #[cfg(feature = "bybe")]
            Self::WebView => Icon::Globe,
            Self::Settings => Icon::Gear,
            Self::LogViewer => Icon::Document,
        }
    }

    const fn description(self) -> &'static str {
        match self {
            Self::Visualizer => "Controller, audio player and information tracker",
            #[cfg(feature = "bybe")]
            Self::WebView => "BYBE web: shop & encounter builder",
            Self::Settings => "Folders, refresh rate and audio output",
            Self::LogViewer => "Application log",
        }
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

const NUMBER_KEYS: [Key; 9] = [
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
];

fn tab_shortcut(index: usize) -> Option<KeyboardShortcut> {
    NUMBER_KEYS
        .get(index)
        .map(|key| KeyboardShortcut::new(Modifiers::COMMAND, *key))
}

const fn output_short_label(kind: OutputKind) -> &'static str {
    match kind {
        OutputKind::Local => "PC audio",
        OutputKind::Discord => "Discord",
    }
}

fn pixel_font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("Pixelify".into()))
}

fn paint_tab(ui: &mut Ui, tab: CurrentTab, selected: bool) -> Response {
    let galley = ui.painter().layout_no_wrap(
        tab.label().to_owned(),
        FontId::proportional(TAB_FONT_SIZE),
        Color32::PLACEHOLDER,
    );
    let content_w = TAB_ICON_SIZE + TAB_ICON_GAP + galley.size().x;
    let size = Vec2::new(2.0f32.mul_add(TAB_PAD_X, content_w), ui.available_height());
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    let hover_text = tab_shortcut(tab.index()).map_or_else(
        || tab.description().to_owned(),
        |s| format!("{}\n{}", tab.description(), ui.ctx().format_shortcut(&s)),
    );
    let response = response
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(hover_text);
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, selected, tab.label())
    });

    if ui.is_rect_visible(rect) {
        let hover_t = ui.ctx().animate_bool_with_time(
            response.id.with("hover"),
            response.hovered(),
            HOVER_ANIM_SECS,
        );
        let painter = ui.painter();
        if !selected && hover_t > 0.0 {
            painter.rect_filled(
                rect.shrink2(Vec2::new(2.0, 8.0)),
                6.0,
                Color32::from_white_alpha(10).gamma_multiply(hover_t),
            );
        }
        let color = if selected {
            theme::TEXT
        } else {
            theme::TEXT_MUTED.lerp_to_gamma(theme::TEXT_HOVER, hover_t)
        };
        let left = rect.center().x - content_w / 2.0;
        let icon_rect = Rect::from_center_size(
            egui::pos2(left + TAB_ICON_SIZE / 2.0, rect.center().y),
            Vec2::splat(TAB_ICON_SIZE),
        );
        icons::paint(painter, tab.icon(), icon_rect, color);
        painter.galley(
            egui::pos2(
                left + TAB_ICON_SIZE + TAB_ICON_GAP,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            color,
        );
    }
    response
}

fn paint_tab_indicator(ui: &Ui, strip_origin: f32, active: Rect) {
    let ctx = ui.ctx();
    let id = Id::new("teatro_tab_indicator");
    let offset = ctx.animate_value_with_time(
        id.with("x"),
        active.min.x - strip_origin,
        INDICATOR_ANIM_SECS,
    );
    let width = ctx.animate_value_with_time(id.with("w"), active.width(), INDICATOR_ANIM_SECS);

    let x = strip_origin + offset + INDICATOR_INSET;
    let w = 2.0f32.mul_add(-INDICATOR_INSET, width).max(0.0);
    let bar = Rect::from_min_size(
        egui::pos2(x, active.max.y - INDICATOR_HEIGHT),
        Vec2::new(w, INDICATOR_HEIGHT),
    );
    let painter = ui.painter();
    painter.rect_filled(
        bar.expand2(Vec2::new(3.0, 1.5)),
        3.0,
        theme::ACCENT.gamma_multiply(0.2),
    );
    painter.rect_filled(bar, INDICATOR_HEIGHT / 2.0, theme::ACCENT);
}

fn paint_vertical_divider(ui: &mut Ui, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, theme::BORDER),
    );
}

impl AkaiVisualizer {
    pub(crate) fn handle_tab_shortcuts(&mut self, ui: &Ui) {
        let tabs = CurrentTab::ALL;
        let current = self.current_tab.index();
        let target = ui.ctx().input_mut(|i| {
            if i.consume_shortcut(&KeyboardShortcut::new(
                Modifiers::COMMAND | Modifiers::SHIFT,
                Key::Tab,
            )) {
                return Some((current + tabs.len() - 1) % tabs.len());
            }
            if i.consume_shortcut(&KeyboardShortcut::new(Modifiers::COMMAND, Key::Tab)) {
                return Some((current + 1) % tabs.len());
            }
            (0..tabs.len()).find(|&idx| tab_shortcut(idx).is_some_and(|s| i.consume_shortcut(&s)))
        });
        if let Some(tab) = target.and_then(|idx| tabs.get(idx)) {
            self.current_tab = *tab;
        }
    }

    pub(crate) fn show_header(&mut self, ui: &mut Ui) {
        Panel::top("teatro_header")
            .exact_size(HEADER_HEIGHT)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(theme::HEADER_BG)
                    .inner_margin(Margin::symmetric(SIDE_MARGIN, 0)),
            )
            .show(ui, |ui| {
                let full = ui
                    .max_rect()
                    .expand2(Vec2::new(f32::from(SIDE_MARGIN), 0.0));
                ui.painter().hline(
                    full.x_range(),
                    full.max.y - 0.5,
                    Stroke::new(1.0, theme::BORDER),
                );

                ui.horizontal_centered(|ui| {
                    Self::draw_brand(ui);
                    ui.add_space(14.0);
                    paint_vertical_divider(ui, 22.0);
                    ui.add_space(6.0);
                    self.draw_tab_strip(ui);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.draw_now_playing(ui);
                    });
                });
            });
    }

    fn draw_brand(ui: &mut Ui) {
        ui.label(
            egui::RichText::new("Teatro")
                .font(pixel_font(22.0))
                .color(theme::ACCENT),
        )
        .on_hover_text("Teatro - TTRPG Helper");
        ui.label(
            egui::RichText::new("by RakuJa")
                .font(pixel_font(11.0))
                .color(theme::TEXT_FAINT),
        );
    }

    fn draw_tab_strip(&mut self, ui: &mut Ui) {
        ui.spacing_mut().item_spacing.x = 2.0;
        let mut strip_origin = None;
        let mut active_rect = None;
        for &tab in CurrentTab::ALL {
            let selected = self.current_tab == tab;
            let response = paint_tab(ui, tab, selected);
            strip_origin.get_or_insert(response.rect.min.x);
            if response.clicked() {
                self.current_tab = tab;
            }
            if selected {
                active_rect = Some(response.rect);
            }
        }
        if let (Some(origin), Some(active)) = (strip_origin, active_rect) {
            paint_tab_indicator(ui, origin, active);
        }
    }

    /// Returns `(title, is_playing)` of the current track, if any.
    fn now_playing(&self) -> Option<(String, bool)> {
        self.gui_data.lock().ok().and_then(|gui_data| {
            let playlist = gui_data.data.current_playlist.as_ref()?;
            let track = playlist.tracks.get(playlist.current_track as usize)?;
            Some((
                title_from_path(&track.file_path),
                gui_data.player_info.is_playing(),
            ))
        })
    }

    /// Compact chip so the current track
    fn draw_now_playing(&mut self, ui: &mut Ui) {
        let Some((title, is_playing)) = self.now_playing() else {
            ui.label(
                egui::RichText::new("Nothing playing")
                    .size(12.0)
                    .color(theme::TEXT_FAINT),
            );
            return;
        };

        let mut job =
            LayoutJob::simple_singleline(title, FontId::proportional(13.0), Color32::PLACEHOLDER);
        job.wrap = TextWrapping::truncate_at_width(NOW_PLAYING_MAX_WIDTH);
        let galley = ui.painter().layout_job(job);

        let dot_radius = 4.0;
        let pad = Vec2::new(10.0, 5.0);
        let gap = 8.0;
        let size = Vec2::new(
            pad.x
                .mul_add(2.0, 2.0f32.mul_add(dot_radius, gap) + galley.size().x),
            pad.y.mul_add(2.0, galley.size().y),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let response = response
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(if is_playing {
                "Now playing - click to open the player"
            } else {
                "Paused - click to open the player"
            });
        if response.clicked() {
            self.current_tab = CurrentTab::Visualizer;
        }

        let painter = ui.painter();
        let bg = if response.hovered() {
            theme::CHIP_BG_HOVER
        } else {
            theme::CHIP_BG
        };
        painter.rect_filled(rect, rect.height() / 2.0, bg);
        let dot_center = egui::pos2(rect.min.x + pad.x + dot_radius, rect.center().y);
        let (dot_color, text_color) = if is_playing {
            (theme::ACCENT, theme::TEXT)
        } else {
            (theme::TEXT_FAINT, theme::TEXT_MUTED)
        };
        painter.circle_filled(dot_center, dot_radius, dot_color);
        painter.galley(
            egui::pos2(
                dot_center.x + dot_radius + gap,
                rect.center().y - galley.size().y / 2.0,
            ),
            galley,
            text_color,
        );
    }

    pub(crate) fn show_status_bar(&self, ui: &mut Ui) {
        let (music_folder, output) = self
            .gui_data
            .lock()
            .ok()
            .and_then(|g| {
                g.data
                    .settings_data
                    .lock()
                    .ok()
                    .map(|s| (s.music_folder.clone(), s.audio_output))
            })
            .unwrap_or_default();

        Panel::bottom("teatro_status")
            .exact_size(STATUS_HEIGHT)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(theme::STATUS_BG)
                    .inner_margin(Margin::symmetric(SIDE_MARGIN, 0)),
            )
            .show(ui, |ui| {
                let full = ui
                    .max_rect()
                    .expand2(Vec2::new(f32::from(SIDE_MARGIN), 0.0));
                ui.painter().hline(
                    full.x_range(),
                    full.min.y + 0.5,
                    Stroke::new(1.0, theme::BORDER),
                );

                let small = |text: String, color: Color32| {
                    egui::RichText::new(text).size(11.5).color(color)
                };
                ui.horizontal_centered(|ui| {
                    ui.label(small("Music folder".into(), theme::TEXT_FAINT));
                    ui.add(egui::Label::new(small(music_folder, theme::TEXT_MUTED)).truncate());

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(small(
                            format!("v{}", env!("CARGO_PKG_VERSION")),
                            theme::TEXT_FAINT,
                        ));
                        paint_vertical_divider(ui, 12.0);
                        ui.label(small(output_short_label(output).into(), theme::TEXT_MUTED));
                        ui.label(small("Output".into(), theme::TEXT_FAINT));
                    });
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_tab_has_a_shortcut() {
        for (i, tab) in CurrentTab::ALL.iter().enumerate() {
            assert_eq!(tab.index(), i);
            assert!(tab_shortcut(i).is_some(), "{tab:?} has no shortcut");
        }
    }
}
