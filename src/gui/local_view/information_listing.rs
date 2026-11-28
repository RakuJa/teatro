use crate::gui::icons::{self, Icon};
use crate::gui::theme;
use crate::gui::ui::AkaiVisualizer;
use crate::states::information_data::InformationEntry;
use egui::{
    Align, Button, Color32, CursorIcon, Frame, Key, Label, Layout, Margin, RichText, ScrollArea,
    Sense, Stroke, TextEdit, Ui, Vec2,
};
use tracing::warn;

const ROW_CONTROLS_W: f32 = 74.0;
const ICON_BUTTON_SIZE: f32 = 20.0;
const INDEX_W: f32 = 20.0;
const ROW_BG: Color32 = Color32::from_rgb(36, 36, 46);
const ROW_BG_HOVER: Color32 = Color32::from_rgb(42, 42, 54);

enum ListAction {
    MoveUp(usize),
    MoveDown(usize),
    Delete(usize),
    StartEditing(usize),
    StopEditing,
    Add(String),
}

fn icon_button(
    ui: &mut Ui,
    icon: Icon,
    tooltip: &str,
    enabled: bool,
    danger: bool,
    row_hovered: bool,
) -> bool {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(ICON_BUTTON_SIZE), sense);
    let response = if enabled {
        response
            .on_hover_cursor(CursorIcon::PointingHand)
            .on_hover_text(tooltip)
    } else {
        response
    };

    let hovered = enabled && response.hovered();
    let color = if !enabled {
        theme::BORDER_STRONG
    } else if hovered && danger {
        theme::ERROR
    } else if row_hovered {
        theme::TEXT_HOVER
    } else {
        theme::TEXT_FAINT
    };
    if hovered {
        ui.painter().rect_filled(
            rect,
            4.0,
            if danger {
                theme::ERROR.gamma_multiply(0.2)
            } else {
                Color32::from_white_alpha(14)
            },
        );
    }
    icons::paint(ui.painter(), icon, rect.shrink(4.0), color);
    response.clicked()
}

fn draw_row(
    ui: &mut Ui,
    index: usize,
    total: usize,
    text: &mut String,
    editing: bool,
    focus_editor: &mut bool,
) -> Vec<ListAction> {
    let mut actions = Vec::new();
    let row_id = ui.id().with(("tracker_row", index));
    let was_hovered = ui.data(|d| d.get_temp::<bool>(row_id).unwrap_or(false));

    let frame = Frame::new()
        .fill(if was_hovered || editing {
            ROW_BG_HOVER
        } else {
            ROW_BG
        })
        .stroke(Stroke::new(
            1.0,
            if editing {
                theme::ACCENT
            } else {
                theme::BORDER
            },
        ))
        .corner_radius(6)
        .inner_margin(Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                ui.add_sized(
                    Vec2::new(INDEX_W, 20.0),
                    Label::new(
                        RichText::new(format!("{}", index + 1))
                            .size(12.0)
                            .color(theme::TEXT_FAINT),
                    ),
                );

                let text_w = (ui.available_width() - ROW_CONTROLS_W).max(40.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(text_w, 0.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(text_w);
                        if editing {
                            let response = ui.add(
                                TextEdit::multiline(text)
                                    .desired_rows(1)
                                    .desired_width(f32::INFINITY)
                                    .background_color(theme::INPUT_BG)
                                    .return_key(egui::KeyboardShortcut::new(
                                        egui::Modifiers::SHIFT,
                                        Key::Enter,
                                    )),
                            );
                            if std::mem::take(focus_editor) {
                                response.request_focus();
                            }
                            let enter =
                                ui.input(|i| i.key_pressed(Key::Enter) && !i.modifiers.shift);
                            if response.lost_focus() || (response.has_focus() && enter) {
                                actions.push(ListAction::StopEditing);
                            }
                        } else {
                            let response = ui
                                .add(
                                    Label::new(
                                        RichText::new(text.as_str()).size(13.5).color(theme::TEXT),
                                    )
                                    .wrap()
                                    .sense(Sense::click()),
                                )
                                .on_hover_cursor(CursorIcon::Text)
                                .on_hover_text("Double-click to edit");
                            if response.double_clicked() {
                                actions.push(ListAction::StartEditing(index));
                            }
                        }
                    },
                );

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if icon_button(ui, Icon::Trash, "Delete", true, true, was_hovered) {
                        actions.push(ListAction::Delete(index));
                    }
                    if icon_button(
                        ui,
                        Icon::ArrowDown,
                        "Move down",
                        index + 1 < total,
                        false,
                        was_hovered,
                    ) {
                        actions.push(ListAction::MoveDown(index));
                    }
                    if icon_button(ui, Icon::ArrowUp, "Move up", index > 0, false, was_hovered) {
                        actions.push(ListAction::MoveUp(index));
                    }
                });
            });
        });

    let hovered = ui.rect_contains_pointer(frame.response.rect);
    if hovered != was_hovered {
        ui.data_mut(|d| d.insert_temp(row_id, hovered));
        ui.ctx().request_repaint();
    }
    actions
}

impl AkaiVisualizer {
    pub(crate) fn draw_information_list(&mut self, ui: &mut Ui) {
        let mut actions = Vec::new();

        Frame::new()
            .fill(theme::CARD_BG)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(10)
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_min_height(ui.available_height());

                self.draw_tracker_header(ui);
                ui.add_space(10.0);
                actions.extend(self.draw_add_row(ui));
                ui.add_space(10.0);

                if self.info_panel_data.information_list.is_empty() {
                    ui.add_space(24.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Nothing tracked yet")
                                .size(13.5)
                                .color(theme::TEXT_MUTED),
                        );
                        ui.label(
                            RichText::new("Initiative order, NPC names, loot...")
                                .size(12.0)
                                .color(theme::TEXT_FAINT),
                        );
                    });
                    return;
                }

                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.spacing_mut().item_spacing.y = 6.0;
                        let data = &mut self.info_panel_data;
                        let total = data.information_list.len();
                        for i in 0..total {
                            let editing = data.editing_index == Some(i);
                            actions.extend(draw_row(
                                ui,
                                i,
                                total,
                                &mut data.information_list[i],
                                editing,
                                &mut data.focus_editor,
                            ));
                        }
                    });
            });

        if self.apply_list_actions(actions) {
            self.persist_information_list();
        }
    }

    fn draw_tracker_header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Information tracker")
                    .size(16.0)
                    .strong()
                    .color(theme::TEXT),
            );
            let count = self.info_panel_data.information_list.len();
            if count > 0 {
                ui.label(
                    RichText::new(count.to_string())
                        .size(12.0)
                        .color(theme::TEXT_FAINT),
                );
            }
        });
        ui.label(
            RichText::new("Double-click an entry to edit it. Saved automatically.")
                .size(11.5)
                .color(theme::TEXT_FAINT),
        );
    }

    fn draw_add_row(&mut self, ui: &mut Ui) -> Option<ListAction> {
        let height = 28.0;
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), height),
            Layout::right_to_left(Align::Center),
            |ui| {
                let can_add = !self.info_panel_data.new_entry.trim().is_empty();
                let add_clicked = ui
                    .add_enabled(
                        can_add,
                        Button::new(RichText::new("Add").strong().color(if can_add {
                            theme::ON_ACCENT
                        } else {
                            theme::TEXT_FAINT
                        }))
                        .fill(if can_add {
                            theme::ACCENT
                        } else {
                            theme::CARD_BG
                        })
                        .corner_radius(6)
                        .min_size(Vec2::new(56.0, height)),
                    )
                    .clicked();
                let response = ui.add(
                    TextEdit::singleline(&mut self.info_panel_data.new_entry)
                        .hint_text("New entry, then Enter")
                        .desired_width(f32::INFINITY)
                        .background_color(theme::INPUT_BG)
                        .margin(Margin::symmetric(8, 6)),
                );
                let entered = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                if (add_clicked || entered) && can_add {
                    // Keep focus in the field for quick consecutive entries.
                    response.request_focus();
                    Some(ListAction::Add(
                        std::mem::take(&mut self.info_panel_data.new_entry)
                            .trim()
                            .to_owned(),
                    ))
                } else {
                    None
                }
            },
        )
        .inner
    }

    fn persist_information_list(&self) {
        let info = self
            .info_panel_data
            .information_list
            .iter()
            .enumerate()
            .map(|(i, s)| InformationEntry {
                position: i,
                data: s.clone(),
            })
            .collect();
        if let Err(e) = InformationEntry::write_to_file(&self.info_panel_data.data_file_path, &info)
        {
            warn!("Failed to write data to file: {e}");
        }
    }

    /// Applies the actions and returns whether the list must be persisted.
    fn apply_list_actions(&mut self, actions: Vec<ListAction>) -> bool {
        let data = &mut self.info_panel_data;
        let mut has_changed = false;

        for action in actions {
            match action {
                ListAction::StartEditing(idx) => {
                    data.editing_index = Some(idx);
                    data.focus_editor = true;
                }
                ListAction::StopEditing => {
                    data.editing_index = None;
                    has_changed = true;
                }
                ListAction::Add(text) => {
                    data.information_list.push(text);
                    has_changed = true;
                }
                ListAction::Delete(idx) => {
                    if idx < data.information_list.len() {
                        data.information_list.remove(idx);
                        data.editing_index = match data.editing_index {
                            Some(e) if e == idx => None,
                            Some(e) if e > idx => Some(e - 1),
                            other => other,
                        };
                        has_changed = true;
                    }
                }
                ListAction::MoveUp(idx) => {
                    if idx > 0 && idx < data.information_list.len() {
                        data.information_list.swap(idx, idx - 1);
                        data.editing_index = None;
                        has_changed = true;
                    }
                }
                ListAction::MoveDown(idx) => {
                    if idx + 1 < data.information_list.len() {
                        data.information_list.swap(idx, idx + 1);
                        data.editing_index = None;
                        has_changed = true;
                    }
                }
            }
        }
        has_changed
    }
}
