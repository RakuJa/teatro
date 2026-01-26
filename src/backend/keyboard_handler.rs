use crate::backend::hw_handler::MidiHandler;
use crate::os_explorer::explorer::find_file_with_prefix;
use crate::states::audio_sinks::AudioSinks;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::RuntimeData;
use anyhow::bail;
use flume::Sender;
use log::{debug, warn};
use ramidier::enums::input_group::KeyboardInputGroup;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use std::sync::{Arc, Mutex};

const fn is_ambience_key(k: u8) -> bool {
    matches!(k, 2 | 4 | 7 | 9 | 11 | 14 | 16 | 19 | 21 | 23)
}

const fn map_key_to_black_key_index(key: u8) -> u8 {
    match key {
        2 => 1,
        4 => 2,
        7 => 3,
        9 => 4,
        11 => 5,
        14 => 6,
        16 => 7,
        19 => 8,
        21 => 9,
        23 => 10,
        _ => 0,
    }
}

const fn map_key_to_white_key_index(key: u8) -> u8 {
    match key {
        1 => 1,
        3 => 2,
        5 => 3,
        6 => 4,
        8 => 5,
        10 => 6,
        12 => 7,
        13 => 8,
        15 => 9,
        17 => 10,
        18 => 11,
        20 => 12,
        22 => 13,
        24 => 14,
        25 => 15,
        _ => 0,
    }
}

#[derive(Debug)]
pub struct KeyboardHandler;

impl MidiHandler for KeyboardHandler {
    type Group = KeyboardInputGroup;

    type State = SoundState;

    fn refresh(
        stale_data: &RuntimeData,
        _tx_data: &Sender<RuntimeData>,
        _audio_sinks: &AudioSinks,
    ) -> RuntimeData {
        stale_data.clone()
    }

    fn listener(
        _midi_out: Arc<Mutex<ChannelOutput>>,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    ) {
        debug!("{stamp}: {msg:?}");
        if msg.value != 0 {
            Self::handle_input(msg.input_group, state);
        }
    }
}

impl KeyboardHandler {
    pub fn handle_input(input_group: KeyboardInputGroup, state: &SoundState) {
        match input_group {
            KeyboardInputGroup::Key(k) => {
                if let Ok(data) = state.data.lock() {
                    if let Ok(audio_sinks) = state.audio_sinks.lock() {
                        if let Err(e) = Self::play_sound_file(k, &data, &audio_sinks, state) {
                            warn!("Error while trying to play sound file: {e}");
                        }
                    }
                }
            }
        }
    }

    fn play_sound_file(
        key: u8,
        data: &RuntimeData,
        audio_sinks: &AudioSinks,
        state: &SoundState,
    ) -> anyhow::Result<()> {
        let w_k = map_key_to_white_key_index(key);
        let (index, folder, filter, volume) = if w_k > 0 {
            (
                w_k,
                data.settings_data.lock().map_or_else(
                    |_| "sound_effect".to_string(),
                    |x| x.sound_effect_folder.clone(),
                ),
                state.sound_effect_filter.clone(),
                data.get_sound_effect_volume(),
            )
        } else {
            let b_k = map_key_to_black_key_index(key);
            if b_k == 0 {
                bail!("Not a valid keyboard key {key}")
            }
            (
                b_k,
                data.settings_data
                    .lock()
                    .map_or_else(|_| "ambience".to_string(), |x| x.ambience_folder.clone()),
                state.ambience_filter.clone(),
                data.get_ambience_volume(),
            )
        };
        let prefix = format!("{index:02}_");
        if let Some(file_path) = find_file_with_prefix(&folder, &prefix) {
            if let Some(file_str) = file_path.to_str() {
                let queue = if is_ambience_key(key) {
                    &audio_sinks.ambience_queue
                } else {
                    &audio_sinks.sound_effect_queue
                };
                Self::play_song(&[file_str.to_string()], queue, &filter, volume);
                Ok(())
            } else {
                bail!("Invalid UTF-8 in file path")
            }
        } else {
            bail!("No audio for key {key} in {folder}");
        }
    }
}
