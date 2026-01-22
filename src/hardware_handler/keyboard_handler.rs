use crate::hardware_handler::midi_handler::MidiHandler;
use crate::os_explorer::explorer::find_file_with_prefix;
use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::AkaiData;
use flume::Sender;
use log::{debug, warn};
use ramidier::enums::input_group::KeyboardInputGroup;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use std::sync::{Arc, Mutex};

const fn is_ambience_key(k: u8) -> bool {
    matches!(k, 2 | 4 | 7 | 9 | 11 | 14 | 16 | 19 | 21 | 23)
}

#[derive(Debug)]
pub struct KeyboardHandler;

impl MidiHandler for KeyboardHandler {
    type Group = KeyboardInputGroup;

    type State = SoundState;

    fn refresh(
        _stale_data: &AkaiData,
        _tx_data: &Sender<AkaiData>,
        _audio_sinks: &AudioSinks,
    ) -> AkaiData {
        _stale_data.clone()
    }

    fn listener(
        _midi_out: &mut ChannelOutput,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    ) {
        debug!("{stamp}: {msg:?}");
        if msg.value == 0 {
            return;
        }
        match msg.input_group {
            KeyboardInputGroup::Key(k) => {
                if let Ok(folder) = state.data.lock().map(|x| x.sound_folder.clone()) {
                    if let Ok(volume) = state.data.lock().map(|x| x.get_sound_effect_volume()) {
                        if let Ok(audio_sinks) = state.audio_sinks.lock() {
                            if let Err(e) = Self::play_sound_file(
                                k,
                                &folder,
                                &audio_sinks,
                                &state.sound_filter,
                                volume,
                            ) {
                                warn!("Error while trying to play sound file: {e}");
                            }
                        }
                    }
                }
            }
        }
    }
}

impl KeyboardHandler {
    fn play_sound_file(
        key: u8,
        sound_folder: &str,
        audio_sinks: &AudioSinks,
        sound_filter: &Arc<Mutex<FilterData>>,
        volume: Option<f32>,
    ) -> Result<(), String> {
        let prefix = format!("{key:02}_");
        let file_path = find_file_with_prefix(sound_folder, &prefix)
            .ok_or_else(|| format!("No audio for key {key} in {sound_folder}"))?;

        let file_str = file_path.to_str().ok_or("Invalid UTF-8 in file path")?;

        let queue = if is_ambience_key(key) {
            &audio_sinks.ambience_queue
        } else {
            &audio_sinks.sound_effect_queue
        };

        Self::play_song(&[file_str.to_string()], queue, sound_filter, volume);
        Ok(())
    }
}
