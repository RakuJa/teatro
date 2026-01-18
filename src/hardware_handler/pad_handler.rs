use crate::MusicState;
use crate::audio::playback_handler;
use crate::hardware_handler::midi_handler::{play_song, update_gui};
use crate::os_explorer::explorer::search_files_in_path;
use biquad::Type;
use bitflags::bitflags;
use log::{debug, info, warn};
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::input_group::PadsAndKnobsInputGroup;
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::io::input_data::MidiInputData;
use ramidier::io::output::ChannelOutput;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct ToggleStates: u16 {
        const CLIP_STOP = 1 << 0;
        const SOLO      = 1 << 1;
        const MUTE      = 1 << 2;
        const REC_ARM   = 1 << 3;
        const SELECT    = 1 << 4;
        const STOP_ALL  = 1 << 5;
        const VOLUME    = 1 << 6;
        const PAN       = 1 << 7;
        const SEND      = 1 << 8;
        const DEVICE    = 1 << 9;
        const SHIFT     = 1 << 10;
        const FILTER    = 1 << 11;
        const START     = 1 << 12;
    }
}

const KNOB_INCREMENT: f32 = 0.005;

impl ToggleStates {
    pub fn toggle_button<T: Into<u8> + Copy>(
        &mut self,
        button: Self,
        midi_out: &mut ChannelOutput,
        key: T,
        color: LedColor,
    ) {
        self.toggle(button);
        change_button_status(midi_out, self.contains(button), key.into(), color);
    }
}

fn handle_pad(pad: PadKey, state: &mut MusicState, midi_out: &mut ChannelOutput) {
    let note = pad.get_index();

    // Turn off previous pad
    if let Some(l_p) = state.data.last_pad_pressed {
        let _ = midi_out.set_pad_led(LedMode::On10Percent, l_p, LedColor::Off);
    }

    state.data.last_pad_pressed = Some(note);
    let prefix = format!("{note:02}_");
    if let Ok(res) = search_files_in_path(state.data.music_folder.as_str(), prefix.as_str()) {
        info!("playing the following audio folder: {}", res.0.display());
        let files = res
            .1
            .iter()
            .filter_map(|x| x.to_str())
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        match &state.audio_sinks.lock() {
            Ok(audio_sinks) => {
                play_song(&files, &audio_sinks.music_queue, &state.music_filter);
            }
            _ => warn!("Failed to get audio sink lock, cannot play song"),
        }
    } else {
        warn!("No folder associated with the given button {note}");
    }
    let color: LedColor = (pad.get_index() + 1).try_into().unwrap_or(LedColor::Green);
    let _ = midi_out.set_pad_led(LedMode::On100Percent, note, color);
}

fn handle_knob(index: u8, value: u8, state: &mut MusicState) {
    let delta = if value > 63 { -1.0 } else { 1.0 };
    match index {
        1 => {
            if !state.data.button_states.contains(ToggleStates::MUTE) {
                match &state.audio_sinks.lock() {
                    Ok(audio_sinks) => {
                        playback_handler::increase_volume(
                            &audio_sinks.music_queue,
                            delta * KNOB_INCREMENT,
                        );
                    }
                    _ => {
                        warn!("Failed to get audio sink lock, could not change volume");
                    }
                }
            }
        }
        2 => playback_handler::change_filter_frequency_value(
            &state.music_filter,
            delta,
            Type::LowPass,
        ),
        3 => playback_handler::change_filter_frequency_value(
            &state.music_filter,
            delta,
            Type::HighPass,
        ),
        4 => playback_handler::change_filter_frequency_value(
            &state.music_filter,
            delta,
            Type::SinglePoleLowPassApprox,
        ),
        5 => match &state.audio_sinks.lock() {
            Ok(audio_sinks) => {
                playback_handler::increase_volume(
                    &audio_sinks.ambience_queue,
                    delta * KNOB_INCREMENT,
                );
            }
            _ => {
                warn!("Could not get audio sinks lock");
            }
        },
        6 => playback_handler::change_filter_frequency_value(
            &state.sound_filter,
            delta,
            Type::LowPass,
        ),
        7 => playback_handler::change_filter_frequency_value(
            &state.sound_filter,
            delta,
            Type::HighPass,
        ),
        8 => playback_handler::change_filter_frequency_value(
            &state.sound_filter,
            delta,
            Type::SinglePoleLowPassApprox,
        ),
        _ => {}
    }
    if !state.data.button_states.contains(ToggleStates::MUTE) {
        state.data.knob_values.entry(index).and_modify(|v| {
            *v += delta * KNOB_INCREMENT;
            *v = v.clamp(0.0, 1.0);
        });
    }
}

fn handle_resume_pause(data: &mut MusicState, midi_out: &mut ChannelOutput) {
    data.data.button_states.toggle_button(
        ToggleStates::FILTER,
        midi_out,
        PadsAndKnobsInputGroup::ResumePause,
        LedColor::Green,
    );

    let filter_type = if data.data.button_states.contains(ToggleStates::FILTER) {
        playback_handler::change_filter_frequency_value(&data.music_filter, 1., Type::LowPass);
        Type::LowPass
    } else {
        playback_handler::change_filter_frequency_value(&data.music_filter, 0., Type::AllPass);
        Type::AllPass
    };

    playback_handler::change_filter_frequency_value(&data.music_filter, 1., filter_type);
}

fn handle_soft_key(key: SoftKey, state: &mut MusicState, midi_out: &mut ChannelOutput) {
    match key {
        SoftKey::ClipStop => state.data.button_states.toggle_button(
            ToggleStates::CLIP_STOP,
            midi_out,
            key,
            LedColor::Green,
        ),
        SoftKey::Solo => {
            state.data.button_states.toggle_button(
                ToggleStates::SOLO,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        SoftKey::Mute => {
            state.data.button_states.toggle_button(
                ToggleStates::MUTE,
                midi_out,
                key,
                LedColor::Green,
            );
            match state.audio_sinks.lock() {
                Ok(audio_sinks) => {
                    if state.data.button_states.contains(ToggleStates::MUTE) {
                        state
                            .data
                            .knob_values
                            .insert(1u8, audio_sinks.music_queue.volume());
                        playback_handler::change_volume(&audio_sinks.music_queue, 0.);
                    } else {
                        playback_handler::change_volume(
                            &audio_sinks.music_queue,
                            *state.data.knob_values.get(&1u8).unwrap_or(&0f32),
                        );
                    }
                }
                _ => {
                    warn!("Failed to get audio sink lock, cannot mute song");
                }
            }
        }
        SoftKey::RecArm => {
            state.data.button_states.toggle_button(
                ToggleStates::REC_ARM,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        SoftKey::Select => {
            state.data.button_states.toggle_button(
                ToggleStates::SELECT,
                midi_out,
                key,
                LedColor::Green,
            );
        }
    }
}

fn handle_knob_ctrl(key: KnobCtrlKey, data: &mut MusicState, midi_out: &mut ChannelOutput) {
    match key {
        KnobCtrlKey::Volume => {
            data.data.button_states.toggle_button(
                ToggleStates::VOLUME,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        KnobCtrlKey::Pan => {
            data.data.button_states.toggle_button(
                ToggleStates::PAN,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        KnobCtrlKey::Send => {
            data.data.button_states.toggle_button(
                ToggleStates::SEND,
                midi_out,
                key,
                LedColor::Green,
            );
        }
        KnobCtrlKey::Device => {
            data.data.button_states.toggle_button(
                ToggleStates::DEVICE,
                midi_out,
                key,
                LedColor::Green,
            );
        }
    }
}

pub fn pad_listener_logic(
    midi_out: &mut ChannelOutput,
    stamp: u64,
    msg: &MidiInputData<PadsAndKnobsInputGroup>,
    state: &mut MusicState,
) {
    debug!("{stamp}: {msg:?}");
    if msg.value > 0 {
        match msg.input_group {
            PadsAndKnobsInputGroup::Pads(ref pad) => handle_pad(*pad, state, midi_out),
            PadsAndKnobsInputGroup::Knob(index) => handle_knob(index, msg.value, state),
            PadsAndKnobsInputGroup::ResumePause => handle_resume_pause(state, midi_out),
            PadsAndKnobsInputGroup::SoftKeys(ref key) => handle_soft_key(*key, state, midi_out),
            PadsAndKnobsInputGroup::KnobCtrl(ref key) => handle_knob_ctrl(*key, state, midi_out),
            PadsAndKnobsInputGroup::StopAllClips => state.data.button_states.toggle_button(
                ToggleStates::STOP_ALL,
                midi_out,
                PadsAndKnobsInputGroup::StopAllClips,
                LedColor::Green,
            ),
            PadsAndKnobsInputGroup::Shift => state.data.button_states.toggle_button(
                ToggleStates::SHIFT,
                midi_out,
                PadsAndKnobsInputGroup::Shift,
                LedColor::Green,
            ),
            PadsAndKnobsInputGroup::Start => state.data.button_states.toggle_button(
                ToggleStates::START,
                midi_out,
                PadsAndKnobsInputGroup::Start,
                LedColor::Green,
            ),
            PadsAndKnobsInputGroup::Left => {
                change_button_status(
                    midi_out,
                    true,
                    PadsAndKnobsInputGroup::Left,
                    LedColor::Green,
                );
            }
            PadsAndKnobsInputGroup::Right => {
                change_button_status(
                    midi_out,
                    true,
                    PadsAndKnobsInputGroup::Right,
                    LedColor::Green,
                );
                match state.audio_sinks.lock() {
                    Ok(audio_sinks) => audio_sinks.music_queue.skip_one(),
                    Err(_) => {
                        warn!("Failed to get audio sink lock, cannot skip music");
                    }
                }
            }
            PadsAndKnobsInputGroup::Up => {
                change_button_status(midi_out, true, PadsAndKnobsInputGroup::Up, LedColor::Green);
            }
            PadsAndKnobsInputGroup::Down => {
                change_button_status(
                    midi_out,
                    true,
                    PadsAndKnobsInputGroup::Down,
                    LedColor::Green,
                );
            }
        }
    } else {
        match msg.input_group {
            PadsAndKnobsInputGroup::Left => {
                change_button_status(
                    midi_out,
                    false,
                    PadsAndKnobsInputGroup::Left,
                    LedColor::Green,
                );
            }
            PadsAndKnobsInputGroup::Right => {
                change_button_status(
                    midi_out,
                    false,
                    PadsAndKnobsInputGroup::Right,
                    LedColor::Green,
                );
            }
            PadsAndKnobsInputGroup::Up => {
                change_button_status(midi_out, false, PadsAndKnobsInputGroup::Up, LedColor::Green);
            }
            PadsAndKnobsInputGroup::Down => {
                change_button_status(
                    midi_out,
                    false,
                    PadsAndKnobsInputGroup::Down,
                    LedColor::Green,
                );
            }
            _ => {}
        }
    }
    update_gui(&state.tx_data, state.data.clone());
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
