use eframe::emath::Pos2;
use eframe::epaint::{Color32, FontId};
use std::time::Duration;

pub fn is_playlist_open(ctx: &egui::Context) -> bool {
    ctx.data(|d| {
        d.get_temp::<bool>(egui::Id::new("akai_playlist_open"))
            .unwrap_or(false)
    })
}

pub fn set_playlist_open(ctx: &egui::Context, open: bool) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("akai_playlist_open"), open));
}

pub fn title_from_path(file_path: &str) -> String {
    std::path::Path::new(file_path)
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

pub fn draw_text_with_shadow(
    painter: &egui::Painter,
    pos: Pos2,
    align: egui::Align2,
    text: &str,
    font: FontId,
    color: Color32,
    shadow_offset: f32,
) {
    // Shadow
    painter.text(
        Pos2::new(pos.x + shadow_offset, pos.y + shadow_offset),
        align,
        text,
        font.clone(),
        Color32::from_rgba_premultiplied(0, 0, 0, 150),
    );
    painter.text(pos, align, text, font, color);
}

pub fn format_time(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}
