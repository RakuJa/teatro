use crate::hardware_handler::pad_handler::ToggleStates;
use bon::bon;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AkaiData {
    pub music_folder: String,
    pub sound_folder: String,
    pub pad_labels: Vec<String>,
    pub knob_values: HashMap<u8, f32>,
    pub button_states: ToggleStates,
    pub last_pad_pressed: Option<u8>,
}

#[bon]
impl AkaiData {
    #[builder]
    pub fn new(
        music_folder: &str,
        sound_folder: &str,
        pad_labels: Option<Vec<String>>,
        knob_values: Option<HashMap<u8, f32>>,
        button_states: Option<ToggleStates>,
        last_pad_pressed: Option<u8>,
    ) -> Self {
        Self {
            music_folder: music_folder.to_string(),
            sound_folder: sound_folder.to_string(),
            pad_labels: pad_labels.unwrap_or_else(|| vec![String::new(); 40]),
            knob_values: knob_values.unwrap_or_else(|| {
                HashMap::from([
                    (1u8, 1.),
                    (2, 0.),
                    (3, 0.),
                    (4, 0.),
                    (5, 0.),
                    (6, 0.),
                    (7, 0.),
                    (8, 0.),
                ])
            }),
            button_states: button_states.unwrap_or_default(),
            last_pad_pressed,
        }
    }
}
