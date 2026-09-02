use anyhow::bail;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsData {
    pub(crate) music_folder: String,
    pub(crate) ambience_folder: String,
    pub(crate) sound_effect_folder: String,
    pub(crate) detected_display_hz: f32,
    pub(crate) repaint_display_hz: f32,
}

impl Default for SettingsData {
    fn default() -> Self {
        let display_hz = detect_refresh_rate();
        Self {
            music_folder: "music".to_string(),
            ambience_folder: "ambience".to_string(),
            sound_effect_folder: "sound".to_string(),
            detected_display_hz: display_hz,
            repaint_display_hz: display_hz,
        }
    }
}

fn detect_refresh_rate() -> f32 {
    DisplayInfo::all().map_or(60.0, |displays| {
        displays
            .iter()
            .find(|d| d.is_primary)
            .or_else(|| displays.first())
            .map_or(60., |d| d.frequency)
    })
}

impl SettingsData {
    pub fn copy_data(&mut self, new_data: &Self) {
        self.music_folder.clone_from(&new_data.music_folder);
        self.ambience_folder.clone_from(&new_data.ambience_folder);
        self.sound_effect_folder
            .clone_from(&new_data.sound_effect_folder);
    }
    pub fn write_to_config(&self, config_path: &str) -> anyhow::Result<()> {
        let toml_string = toml::to_string(self)?;
        if let Err(e) = fs::write(config_path, &toml_string) {
            bail!("Failed to write settings file: {e}");
        }
        debug!("Settings file has been written");
        Ok(())
    }

    pub fn load_from_config(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}
