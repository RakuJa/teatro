use crate::MusicState;
use crate::audio::playback_handler;
use crate::backend::hw_handler::MidiHandler;
use crate::os_explorer::explorer::{
    get_album_name_from_folder_in_path, map_to_indexed_vec, search_files_in_path,
};
use crate::states::audio_sinks::AudioSinks;
use crate::states::button_states::ToggleStates;
use crate::states::filter_data::FilterData;
use crate::states::knob_value_update::KnobValueUpdate;
use crate::states::visualizer::RuntimeData;
use biquad::Type;
use flume::Sender;
use log::{debug, info, warn};
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::input_group::PadsAndKnobsInputGroup;
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::sync::{Arc, Mutex};

const KNOB_INCREMENT: f32 = 0.005;

impl ToggleStates {
    pub fn toggle_button<T: Into<u8> + Copy>(
        &mut self,
        button: Self,
        midi_out: Option<&mut ChannelOutput>,
        key: T,
        color: LedColor,
    ) {
        self.toggle(button);
        if let Some(out) = midi_out {
            change_button_status(out, self.contains(button), key.into(), color);
        }
    }
}

pub struct PadHandler;
impl MidiHandler for PadHandler {
    type Group = PadsAndKnobsInputGroup;
    type State = MusicState;

    fn refresh(
        old_data: &RuntimeData,
        tx_data: &Sender<RuntimeData>,
        audio_sinks: &AudioSinks,
    ) -> RuntimeData {
        let x = old_data.current_playlist.as_ref().map_or_else(
            || old_data.clone(),
            |playlist| RuntimeData {
                settings_data: old_data.settings_data.clone(),
                pad_labels: old_data.pad_labels.clone(),
                knob_values: old_data.knob_values.clone(),
                button_states: old_data.button_states,
                last_pad_pressed: old_data.last_pad_pressed,
                current_playlist: Some(Self::get_current_playlist_state(
                    playlist.clone(),
                    &audio_sinks.music_queue,
                )),
            },
        );
        Self::update_gui(tx_data, &x);
        x
    }
    fn listener(
        midi_out: Arc<Mutex<ChannelOutput>>,
        stamp: u64,
        msg: &MidiInputData<Self::Group>,
        state: &mut Self::State,
    ) {
        debug!("{stamp}: {msg:?}");
        if let Ok(mut out) = midi_out.lock() {
            if msg.value > 0 {
                Self::handle_input_pressed(Some(&mut *out), msg.input_group, msg.value, state);
            } else {
                Self::handle_input_released(Some(&mut *out), msg.input_group, msg.value, state);
            }
        }
        if let Ok(mut data) = state.data.lock()
            && let Ok(audio_sink) = state.audio_sinks.lock()
        {
            let x = Self::refresh(&data, &state.tx_data, &audio_sink);
            data.copy_data(x);
            Self::update_gui(&state.tx_data, &data);
        }
    }
}

impl PadHandler {
    pub fn get_pad_albums_list(music_folder: &str) -> anyhow::Result<Vec<String>> {
        Ok(
            map_to_indexed_vec(get_album_name_from_folder_in_path(music_folder)?)
                .into_iter()
                .flatten()
                .collect(),
        )
    }

    pub fn update_pad_albums_list(
        stale_data: &RuntimeData,
        tx_data: &Sender<RuntimeData>,
    ) -> anyhow::Result<RuntimeData> {
        let folder = stale_data
            .settings_data
            .lock()
            .map_or_else(|_| "music".to_string(), |x| x.music_folder.clone());
        let data = RuntimeData {
            settings_data: stale_data.settings_data.clone(),
            pad_labels: Self::get_pad_albums_list(&folder)?,
            knob_values: stale_data.knob_values.clone(),
            button_states: stale_data.button_states,
            last_pad_pressed: stale_data.last_pad_pressed,
            current_playlist: stale_data.current_playlist.clone(),
        };
        Self::update_gui(tx_data, &data);
        Ok(data)
    }

    fn toggle_state_button(
        state: &MusicState,
        midi_out: Option<&mut ChannelOutput>,
        toggle_state: ToggleStates,
        input_group: PadsAndKnobsInputGroup,
    ) {
        if let Ok(mut data) = state.data.lock() {
            data.button_states
                .toggle_button(toggle_state, midi_out, input_group, LedColor::Green);
        } else {
            warn!("Could not lock mutex for toggle button: {toggle_state:?}");
        }
    }

    pub fn handle_input_released(
        midi_out: Option<&mut ChannelOutput>,
        input_group: PadsAndKnobsInputGroup,
        _value: u8,
        _state: &MusicState,
    ) {
        match input_group {
            PadsAndKnobsInputGroup::Left
            | PadsAndKnobsInputGroup::Right
            | PadsAndKnobsInputGroup::Up
            | PadsAndKnobsInputGroup::Down => {
                if let Some(out) = midi_out {
                    change_button_status(out, false, input_group, LedColor::Green);
                }
            }
            _ => {}
        }
    }

    pub fn handle_input_pressed(
        midi_out: Option<&mut ChannelOutput>,
        input_group: PadsAndKnobsInputGroup,
        value: u8,
        state: &MusicState,
    ) {
        match input_group {
            PadsAndKnobsInputGroup::Pads(ref pad) => Self::handle_pad(*pad, state, midi_out),
            PadsAndKnobsInputGroup::Knob(index) => {
                Self::handle_knob(index, KnobValueUpdate::from(value), state);
            }
            PadsAndKnobsInputGroup::ResumePause => Self::handle_resume_pause(state, midi_out),
            PadsAndKnobsInputGroup::SoftKeys(key) => Self::handle_soft_key(key, state, midi_out),
            PadsAndKnobsInputGroup::KnobCtrl(key) => {
                Self::handle_knob_ctrl(key, state, midi_out);
            }
            PadsAndKnobsInputGroup::StopAllClips => {
                Self::toggle_state_button(state, midi_out, ToggleStates::STOP_ALL, input_group);
                match state.audio_sinks.lock() {
                    Ok(audio_sinks) => {
                        if let Ok(d) = state.data.lock() {
                            if d.button_states.contains(ToggleStates::STOP_ALL) {
                                playback_handler::pause_track(&audio_sinks.music_queue);
                                playback_handler::pause_track(&audio_sinks.ambience_queue);
                                playback_handler::pause_track(&audio_sinks.sound_effect_queue);
                            } else {
                                if !d.button_states.contains(ToggleStates::CLIP_STOP) {
                                    playback_handler::resume_track(&audio_sinks.music_queue);
                                }
                                playback_handler::resume_track(&audio_sinks.ambience_queue);
                                playback_handler::resume_track(&audio_sinks.sound_effect_queue);
                            }
                        }
                    }

                    _ => {
                        warn!("Failed to get audio sink lock, cannot mute all songs");
                    }
                }
            }
            PadsAndKnobsInputGroup::Shift => {
                Self::toggle_state_button(state, midi_out, ToggleStates::SHIFT, input_group);
            }
            PadsAndKnobsInputGroup::Start => {
                Self::toggle_state_button(state, midi_out, ToggleStates::START, input_group);
            }
            PadsAndKnobsInputGroup::Left
            | PadsAndKnobsInputGroup::Up
            | PadsAndKnobsInputGroup::Down => {
                if let Some(out) = midi_out {
                    change_button_status(out, true, input_group, LedColor::Green);
                }
            }
            PadsAndKnobsInputGroup::Right => {
                if let Some(out) = midi_out {
                    change_button_status(out, true, input_group, LedColor::Green);
                }
                if let Ok(audio_sinks) = state.audio_sinks.lock() {
                    audio_sinks.music_queue.skip_one();
                } else {
                    warn!("Failed to get audio sink lock, cannot skip music");
                }
            }
        }
    }

    fn handle_pad(pad: PadKey, state: &MusicState, midi_out: Option<&mut ChannelOutput>) {
        let note = pad.get_index();
        if let Ok(mut data) = state.data.lock() {
            let old_pad = data.last_pad_pressed;
            data.last_pad_pressed = Some(note);
            let prefix = format!("{note:02}_");
            if let Ok(res) = search_files_in_path(
                data.settings_data
                    .lock()
                    .map_or_else(|_| "music".to_string(), |x| x.music_folder.clone())
                    .as_ref(),
                prefix.as_str(),
            ) {
                info!("playing the following audio folder: {}", res.0.display());
                let mut files = res
                    .1
                    .iter()
                    .filter_map(|x| x.to_str())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>();
                if data.button_states.contains(ToggleStates::SEND) {
                    fastrand::shuffle(files.as_mut_slice());
                }
                if let Ok(audio_sinks) = state.audio_sinks.lock() {
                    data.current_playlist = Self::play_song(
                        &files,
                        &audio_sinks.music_queue,
                        &state.music_filter,
                        data.get_music_volume(),
                    );
                } else {
                    warn!("Failed to get audio sink lock, cannot play song");
                }
            } else {
                warn!("No folder associated with the given button {note}");
            }
            let color: LedColor = (pad.get_index() + 1).try_into().unwrap_or(LedColor::Green);
            if let Some(out) = midi_out {
                // Turn off previous pad
                if let Some(l_p) = old_pad {
                    let _ = out.set_pad_led(LedMode::On10Percent, l_p, LedColor::Off);
                }
                let _ = out.set_pad_led(LedMode::On100Percent, note, color);
            }
        } else {
            warn!("Failed to get a lock on data. Will not handle pad action");
        }
    }

    fn handle_knob(index: u8, value: KnobValueUpdate, state: &MusicState) {
        let delta = value.into();
        if let Ok(mut data) = state.data.lock() {
            match index {
                1 => {
                    if !data.button_states.contains(ToggleStates::MUTE) {
                        adjust_queue_volume(state, |s| &s.music_queue, delta);
                    }
                }
                2..=4 => {
                    let filter_type = match index {
                        2 => Type::LowPass,
                        3 => Type::HighPass,
                        4 => Type::SinglePoleLowPassApprox,
                        _ => unreachable!(
                            "The previous guard should always filter number higher than 4 and lower than 2"
                        ),
                    };
                    adjust_filter(&state.music_filter, delta, filter_type);
                }
                5 => adjust_queue_volume(state, |s| &s.ambience_queue, delta),
                6 => adjust_filter(&state.ambience_filter, delta, Type::LowPass),
                7 => adjust_queue_volume(state, |s| &s.sound_effect_queue, delta),
                8 => adjust_filter(&state.sound_effect_filter, delta, Type::HighPass),
                _ => {}
            }
            if !data.button_states.contains(ToggleStates::MUTE) {
                data.knob_values.entry(index).and_modify(|v| {
                    *v += delta * KNOB_INCREMENT;
                    *v = v.clamp(0.0, 1.0);
                });
            }
        } else {
            warn!("Failed to get a lock on data. Will not handle knob action");
        }
    }

    fn handle_resume_pause(state: &MusicState, midi_out: Option<&mut ChannelOutput>) {
        if let Ok(mut data) = state.data.lock() {
            data.button_states.toggle_button(
                ToggleStates::FILTER,
                midi_out,
                PadsAndKnobsInputGroup::ResumePause,
                LedColor::Green,
            );

            let filter_type = if data.button_states.contains(ToggleStates::FILTER) {
                playback_handler::change_filter_frequency_value(
                    &state.music_filter,
                    1.,
                    Type::LowPass,
                );
                Type::LowPass
            } else {
                playback_handler::change_filter_frequency_value(
                    &state.music_filter,
                    0.,
                    Type::AllPass,
                );
                Type::AllPass
            };

            playback_handler::change_filter_frequency_value(&state.music_filter, 1., filter_type);
        }
    }

    fn handle_soft_key(key: SoftKey, state: &MusicState, midi_out: Option<&mut ChannelOutput>) {
        match key {
            SoftKey::Mute => {
                if let Ok(mut data) = state.data.lock() {
                    data.button_states.toggle_button(
                        ToggleStates::from(key),
                        midi_out,
                        key,
                        LedColor::Green,
                    );
                    match state.audio_sinks.lock() {
                        Ok(audio_sinks) => {
                            if data.button_states.contains(ToggleStates::MUTE) {
                                data.knob_values
                                    .insert(1u8, audio_sinks.music_queue.volume());
                                playback_handler::change_volume(&audio_sinks.music_queue, 0.);
                            } else {
                                playback_handler::change_volume(
                                    &audio_sinks.music_queue,
                                    *data.knob_values.get(&1u8).unwrap_or(&0f32),
                                );
                            }
                        }
                        _ => {
                            warn!("Failed to get audio sink lock, cannot mute song");
                        }
                    }
                } else {
                    warn!("Failed to get data lock, cannot handle mute press");
                }
            }
            SoftKey::ClipStop => {
                if let Ok(mut data) = state.data.lock() {
                    data.button_states.toggle_button(
                        ToggleStates::from(key),
                        midi_out,
                        key,
                        LedColor::Green,
                    );
                }
                if let Ok(audio_sinks) = state.audio_sinks.lock() {
                    if let Ok(d) = state.data.lock() {
                        if d.button_states.contains(ToggleStates::CLIP_STOP) {
                            playback_handler::pause_track(&audio_sinks.music_queue);
                        } else if !d.button_states.contains(ToggleStates::STOP_ALL) {
                            playback_handler::resume_track(&audio_sinks.music_queue);
                        }
                    }
                } else {
                    warn!("Failed to get audio sink lock, cannot mute song");
                }
            }
            SoftKey::Solo => {
                if let Ok(mut data) = state.data.lock() {
                    data.button_states.toggle_button(
                        ToggleStates::from(key),
                        midi_out,
                        key,
                        LedColor::Green,
                    );
                }
                match state.audio_sinks.lock() {
                    Ok(audio_sinks) => {
                        playback_handler::stop_track(&audio_sinks.sound_effect_queue);
                        playback_handler::stop_track(&audio_sinks.ambience_queue);
                    }
                    _ => {
                        warn!("Failed to get audio sink lock, cannot mute song");
                    }
                }
            }
            _ => {
                if let Ok(mut data) = state.data.lock() {
                    data.button_states.toggle_button(
                        ToggleStates::from(key),
                        midi_out,
                        key,
                        LedColor::Green,
                    );
                } else {
                    warn!("Failed to get data lock, cannot handle soft key press");
                }
            }
        }
    }

    fn handle_knob_ctrl(
        key: KnobCtrlKey,
        state: &MusicState,
        midi_out: Option<&mut ChannelOutput>,
    ) {
        if let Ok(mut data) = state.data.lock() {
            data.button_states.toggle_button(
                ToggleStates::from(key),
                midi_out,
                key,
                LedColor::Green,
            );
        }
    }
}

fn adjust_queue_volume(
    state: &MusicState,
    queue_selector: impl FnOnce(&AudioSinks) -> &Sink,
    delta: f32,
) {
    match state.audio_sinks.lock() {
        Ok(audio_sinks) => {
            playback_handler::increase_volume(queue_selector(&audio_sinks), delta * KNOB_INCREMENT);
        }
        Err(_) => warn!("Failed to get audio sink lock, could not change volume"),
    }
}

fn adjust_filter(filter: &Arc<Mutex<FilterData>>, delta: f32, filter_type: Type<f32>) {
    playback_handler::change_filter_frequency_value(filter, delta, filter_type);
}

fn change_button_status<T>(
    midi_out: &mut ChannelOutput,
    next_state: bool,
    button_index: T,
    color: LedColor,
) where
    T: Into<u8>,
{
    let _ = midi_out.set_pad_led(
        LedMode::On100Percent,
        button_index,
        if next_state { color } else { LedColor::Off },
    );
}
