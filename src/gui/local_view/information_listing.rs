use crate::gui::ui::AkaiVisualizer;
use crate::states::information_data::InformationEntry;
use eframe::emath::Rect;
use eframe::epaint::Color32;
use egui::{Frame, Pos2, RichText, ScrollArea, Vec2};
use log::warn;

struct IconButtonStyle {
    size: Vec2,
    enabled_bg: Color32,
    enabled_bg_hover: Color32,
    disabled_bg: Color32,
    enabled_border: Color32,
    disabled_border: Color32,
}

impl IconButtonStyle {
    fn new(scale: f32) -> Self {
        Self {
            size: egui::vec2(28.0 * scale, 20.0 * scale),
            enabled_bg: Color32::from_rgb(60, 80, 120),
            enabled_bg_hover: Color32::from_rgb(70, 90, 140),
            disabled_bg: Color32::from_rgb(45, 45, 55),
            enabled_border: Color32::from_rgb(100, 140, 220),
            disabled_border: Color32::from_rgb(60, 60, 70),
        }
    }

    fn delete_style(scale: f32) -> Self {
        Self {
            size: egui::vec2(24.0 * scale, 24.0 * scale),
            enabled_bg: Color32::from_rgb(120, 50, 50),
            enabled_bg_hover: Color32::from_rgb(180, 60, 60),
            disabled_bg: Color32::from_rgb(45, 45, 55),
            enabled_border: Color32::from_rgb(200, 80, 80),
            disabled_border: Color32::from_rgb(60, 60, 70),
        }
    }
}

#[derive(Copy, Clone)]
enum ArrowDirection {
    Up,
    Down,
}

fn draw_arrow_button(
    ui: &mut egui::Ui,
    style: &IconButtonStyle,
    direction: ArrowDirection,
    enabled: bool,
    scale: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(style.size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let _ = ui.style().interact(&response);
        let bg_color = if enabled {
            if response.hovered() {
                style.enabled_bg_hover
            } else {
                style.enabled_bg
            }
        } else {
            style.disabled_bg
        };

        ui.painter().rect_filled(rect, 4.0, bg_color);
        let border_color = if enabled {
            style.enabled_border
        } else {
            style.disabled_border
        };

        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, border_color),
            egui::epaint::StrokeKind::Outside,
        );
        let arrow_color = if enabled {
            Color32::from_rgb(180, 200, 255)
        } else {
            Color32::from_rgb(80, 80, 90)
        };

        draw_arrow(ui, rect.center(), direction, 6.0 * scale, arrow_color);
    }

    response
}

fn draw_arrow(ui: &egui::Ui, center: Pos2, direction: ArrowDirection, size: f32, color: Color32) {
    let points = match direction {
        ArrowDirection::Up => {
            let top = center + egui::vec2(0.0, -size / 2.0);
            let bottom_left = center + egui::vec2(-size / 1.5, size / 2.0);
            let bottom_right = center + egui::vec2(size / 1.5, size / 2.0);
            vec![top, bottom_left, bottom_right]
        }
        ArrowDirection::Down => {
            let bottom = center + egui::vec2(0.0, size / 2.0);
            let top_left = center + egui::vec2(-size / 1.5, -size / 2.0);
            let top_right = center + egui::vec2(size / 1.5, -size / 2.0);
            vec![bottom, top_left, top_right]
        }
    };

    ui.painter().add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::NONE,
    ));
}

fn draw_delete_button(ui: &mut egui::Ui, style: &IconButtonStyle, scale: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(style.size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let bg_color = if response.hovered() {
            style.enabled_bg_hover
        } else {
            style.enabled_bg
        };

        ui.painter().rect_filled(rect, 4.0, bg_color);

        ui.painter().rect_stroke(
            rect,
            4.0,
            egui::Stroke::new(1.0, style.enabled_border),
            egui::epaint::StrokeKind::Outside,
        );

        let center = rect.center();
        let x_size = 8.0 * scale;
        let x_color = Color32::from_rgb(255, 200, 200);

        ui.painter().line_segment(
            [
                center + egui::vec2(-x_size / 2.0, -x_size / 2.0),
                center + egui::vec2(x_size / 2.0, x_size / 2.0),
            ],
            egui::Stroke::new(2.0 * scale, x_color),
        );

        ui.painter().line_segment(
            [
                center + egui::vec2(x_size / 2.0, -x_size / 2.0),
                center + egui::vec2(-x_size / 2.0, x_size / 2.0),
            ],
            egui::Stroke::new(2.0 * scale, x_color),
        );
    }

    response.on_hover_text("Delete item")
}

fn draw_vertical_divider(ui: &egui::Ui) {
    let divider_rect = Rect::from_min_size(ui.cursor().min, egui::vec2(1.5, ui.available_height()));
    ui.painter()
        .rect_filled(divider_rect, 1.0, Color32::from_rgb(80, 80, 95));
}

fn draw_item_controls(
    ui: &mut egui::Ui,
    index: usize,
    total_items: usize,
    scale: f32,
) -> Vec<ListAction> {
    let mut actions = vec![];
    let style = IconButtonStyle::new(scale);

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;

        let up_enabled = index > 0;
        let up_response = draw_arrow_button(ui, &style, ArrowDirection::Up, up_enabled, scale);
        if up_response.clicked() && up_enabled {
            actions.push(ListAction::MoveUp(index));
        }

        let down_enabled = index < total_items - 1;
        let down_response =
            draw_arrow_button(ui, &style, ArrowDirection::Down, down_enabled, scale);
        if down_response.clicked() && down_enabled {
            actions.push(ListAction::MoveDown(index));
        }
    });

    actions
}

fn draw_item_content(
    ui: &mut egui::Ui,
    index: usize,
    text: &mut String,
    is_editing: bool,
    scale: f32,
) -> Vec<ListAction> {
    let mut actions = vec![];

    if is_editing {
        let edit_response = ui.add(
            egui::TextEdit::singleline(text)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Body)
                .text_color(Color32::from_rgb(240, 240, 250)),
        );

        if edit_response.lost_focus()
            && (ui.input(|i| i.key_pressed(egui::Key::Enter))
                || ui.input(|i| i.key_pressed(egui::Key::Escape)))
        {
            actions.push(ListAction::StopEditing);
        }

        edit_response.request_focus();
    } else {
        let label_response = ui.add(
            egui::Label::new(
                RichText::new(&*text)
                    .color(Color32::from_rgb(220, 220, 235))
                    .size(14.0 * scale),
            )
            .wrap()
            .sense(egui::Sense::click()),
        );

        if label_response.double_clicked() {
            actions.push(ListAction::StartEditing(index));
        }

        if label_response.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        label_response.on_hover_text("Double-click to edit");
    }

    actions
}

enum ListAction {
    MoveUp(usize),
    MoveDown(usize),
    Delete(usize),
    StartEditing(usize),
    StopEditing,
}

fn draw_list_item(
    ui: &mut egui::Ui,
    index: usize,
    text: &mut String,
    is_editing: bool,
    total_items: usize,
    scale: f32,
) -> Vec<ListAction> {
    let mut actions = vec![];

    Frame::new()
        .fill(if is_editing {
            Color32::from_rgb(50, 50, 60)
        } else {
            Color32::from_rgb(40, 40, 50)
        })
        .stroke(egui::Stroke::new(
            1.0,
            if is_editing {
                Color32::from_rgb(100, 150, 255)
            } else {
                Color32::from_rgb(60, 60, 75)
            },
        ))
        .corner_radius(8.0)
        .inner_margin(egui::vec2(12.0, 10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                actions.extend(draw_item_controls(ui, index, total_items, scale));

                ui.add_space(10.0);
                draw_vertical_divider(ui);
                ui.add_space(10.0);

                ui.vertical(|ui| {
                    actions.extend(draw_item_content(ui, index, text, is_editing, scale));
                });

                ui.add_space(10.0);

                ui.vertical(|ui| {
                    ui.add_space(2.0);
                    let delete_style = IconButtonStyle::delete_style(scale);
                    let delete_response = draw_delete_button(ui, &delete_style, scale);
                    if delete_response.clicked() {
                        actions.push(ListAction::Delete(index));
                    }
                });
            });
        });

    actions
}
impl AkaiVisualizer {
    pub(crate) fn draw_information_list(&mut self, ui: &mut egui::Ui, scale: f32) {
        Frame::new()
            .fill(Color32::from_rgb(25, 25, 30))
            .stroke(egui::Stroke::new(1.5, Color32::from_rgb(70, 70, 85)))
            .corner_radius(12.0)
            .inner_margin(16.0)
            .shadow(egui::epaint::Shadow {
                offset: [0, 2],
                blur: 8,
                spread: 0,
                color: Color32::from_black_alpha(40),
            })
            .show(ui, |ui| {
                ui.set_width(280.0 * scale);

                Self::draw_header(ui, scale);

                let mut actions = vec![];

                ScrollArea::vertical()
                    .max_height(400.0 * scale)
                    .show(ui, |ui| {
                        let list_length = self.info_panel_data.information_list.len();
                        for i in 0..list_length {
                            let is_editing = self.info_panel_data.editing_index == Some(i);

                            actions.extend(draw_list_item(
                                ui,
                                i,
                                &mut self.info_panel_data.information_list[i],
                                is_editing,
                                list_length,
                                scale,
                            ));

                            ui.add_space(8.0);
                        }
                    });

                ui.add_space(12.0);

                self.draw_add_button(ui, scale);

                if self.apply_list_actions(actions) {
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
                    if let Err(e) =
                        InformationEntry::write_to_file(&self.info_panel_data.data_file_path, &info)
                    {
                        warn!("Failed to write data to file: {e}");
                    }
                }
            });
    }

    fn draw_header(ui: &mut egui::Ui, scale: f32) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Information Tracker")
                    .size(18.0 * scale)
                    .strong()
                    .color(Color32::from_rgb(230, 230, 240)),
            );
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(8.0);
    }

    fn draw_add_button(&mut self, ui: &mut egui::Ui, scale: f32) {
        let add_button = ui.add(
            egui::Button::new(
                RichText::new("+ Add New Item")
                    .size(14.0 * scale)
                    .color(Color32::from_rgb(200, 220, 255)),
            )
            .fill(Color32::from_rgb(50, 80, 120))
            .stroke(egui::Stroke::new(1.0, Color32::from_rgb(100, 140, 220)))
            .corner_radius(6.0)
            .min_size(egui::vec2(ui.available_width(), 32.0 * scale)),
        );

        if add_button.clicked() {
            let new_pos = self.info_panel_data.information_list.len();
            self.info_panel_data
                .information_list
                .push(String::from("New Item"));
            self.info_panel_data.editing_index = Some(new_pos);
        }
    }

    fn apply_list_actions(&mut self, actions: Vec<ListAction>) -> bool {
        let mut has_changed = actions.is_empty();

        for action in actions {
            match action {
                ListAction::StartEditing(idx) => {
                    has_changed = true;
                    self.info_panel_data.editing_index = Some(idx);
                }

                ListAction::StopEditing => {
                    has_changed = true;
                    self.info_panel_data.editing_index = None;
                }

                ListAction::Delete(idx) => {
                    if !self.info_panel_data.information_list.is_empty() {
                        has_changed = true;
                        self.info_panel_data.information_list.remove(idx);
                        if let Some(editing) = self.info_panel_data.editing_index {
                            if editing == idx {
                                self.info_panel_data.editing_index = None;
                            } else if editing > idx {
                                self.info_panel_data.editing_index = Some(editing - 1);
                            }
                        }
                    }
                }

                ListAction::MoveUp(idx) => {
                    if idx > 0 {
                        has_changed = true;
                        self.info_panel_data.information_list.swap(idx, idx - 1);
                    }
                }

                ListAction::MoveDown(idx) => {
                    if idx
                        < self
                            .info_panel_data
                            .information_list
                            .len()
                            .saturating_sub(1)
                    {
                        has_changed = true;
                        self.info_panel_data.information_list.swap(idx, idx + 1);
                    }
                }
            }
        }
        has_changed
    }
}
