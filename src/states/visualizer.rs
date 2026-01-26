use crate::states::button_states::ToggleStates;
use crate::states::playlist_data::PlaylistData;
use crate::states::settings_data::SettingsData;
use bon::bon;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct RuntimeData {
    pub settings_data: Arc<Mutex<SettingsData>>,
    pub pad_labels: Vec<String>,
    pub knob_values: HashMap<u8, f32>,
    pub button_states: ToggleStates,
    pub last_pad_pressed: Option<u8>,
    pub current_playlist: Option<PlaylistData>,
}

#[bon]
impl RuntimeData {
    #[builder]
    pub fn new(
        settings_data: Arc<Mutex<SettingsData>>,
        pad_labels: Option<Vec<String>>,
        knob_values: Option<HashMap<u8, f32>>,
        button_states: Option<ToggleStates>,
        last_pad_pressed: Option<u8>,
        current_playlist: Option<PlaylistData>,
    ) -> Self {
        Self {
            settings_data,
            pad_labels: pad_labels.unwrap_or_else(|| vec![String::new(); 40]),
            knob_values: knob_values.unwrap_or_else(|| {
                HashMap::from([
                    (1u8, 0.1),
                    (2, 0.),
                    (3, 0.),
                    (4, 0.),
                    (5, 0.1),
                    (6, 0.),
                    (7, 0.1),
                    (8, 0.),
                ])
            }),
            button_states: button_states.unwrap_or_default(),
            last_pad_pressed,
            current_playlist,
        }
    }

    pub fn copy_data(&mut self, new_data: Self) {
        self.button_states = new_data.button_states;
        self.pad_labels = new_data.pad_labels;
        self.button_states = new_data.button_states;
        self.settings_data = new_data.settings_data;
        self.current_playlist = new_data.current_playlist;
        self.last_pad_pressed = new_data.last_pad_pressed;
    }

    pub fn get_music_volume(&self) -> Option<f32> {
        self.knob_values.get(&1).copied()
    }

    pub fn get_ambience_volume(&self) -> Option<f32> {
        self.knob_values.get(&5).copied()
    }
    pub fn get_sound_effect_volume(&self) -> Option<f32> {
        self.knob_values.get(&7).copied()
    }
}
