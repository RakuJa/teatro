use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingsData {
    pub(crate) music_folder: String,
    pub(crate) ambience_folder: String,
    pub(crate) sound_effect_folder: String,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            music_folder: "music".to_string(),
            ambience_folder: "ambience".to_string(),
            sound_effect_folder: "sound".to_string(),
        }
    }
}

impl SettingsData {
    pub fn copy_data(&mut self, new_data: &Self) {
        self.music_folder.clone_from(&new_data.music_folder);
        self.ambience_folder.clone_from(&new_data.ambience_folder);
        self.sound_effect_folder
            .clone_from(&new_data.sound_effect_folder);
    }
    pub fn write_to_config(&self, path: Option<&str>) -> anyhow::Result<()> {
        let config_path = path.map_or_else(
            || env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yml".to_string()),
            ToString::to_string,
        );
        let toml_string = toml::to_string(self)?;
        if matches!(fs::write(config_path, toml_string), Ok(())) {
            bail!("Failed to write settings file");
        }
        Ok(())
    }

    pub fn load_from_config(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}
