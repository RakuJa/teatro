use crate::SoundState;
use crate::hardware_handler::midi_handler::play_song;
use crate::os_explorer::explorer::find_file_with_prefix;
use log::{debug, warn};
use ramidier::enums::input_group::KeyboardInputGroup;
use ramidier::io::input_data::MidiInputData;

const fn is_ambience_key(k: u8) -> bool {
    matches!(k, 2 | 4 | 7 | 9 | 11 | 14 | 16 | 19 | 21 | 23)
}

pub fn keyboard_listener_logic(
    stamp: u64,
    msg: &MidiInputData<KeyboardInputGroup>,
    state: &SoundState,
) {
    debug!("{stamp}: {msg:?}");
    if msg.value == 0 {
        return;
    }
    match msg.input_group {
        KeyboardInputGroup::Key(k) => {
            let prefix = format!("{k:02}_");
            let sound_folder = state.data.sound_folder.clone();

            if let Some(res) = find_file_with_prefix(sound_folder.as_str(), prefix.as_str()) {
                if let Some(files) = res.to_str().map(ToString::to_string) {
                    match state.audio_sinks.lock() {
                        Ok(audio_sinks) => {
                            play_song(
                                &[files],
                                if is_ambience_key(k) {
                                    &audio_sinks.ambience_queue
                                } else {
                                    &audio_sinks.sound_effect_queue
                                },
                                &state.sound_filter,
                            );
                        }
                        _ => {
                            warn!("Failed to get audio sink lock, cannot play audio");
                        }
                    }
                } else {
                    warn!("Failed to parse file path, is UTF-8?, cannot play audio");
                }
            } else {
                warn!("No audio associated with the given key {k} in the folder {sound_folder}");
            }
        }
    }
}
