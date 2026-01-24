use crate::MidiOutputChannels;
use crate::comms::command::{Command, Device};
use crate::gui::ui::GuiData;
use crate::hardware_handler::hw_handler::MidiHandler;
use crate::hardware_handler::keyboard_handler::KeyboardHandler;
use crate::hardware_handler::pad_handler::PadHandler;
use crate::states::button_states::ToggleStates;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::AkaiData;
use flume::{Receiver, Sender};
use log::{debug, warn};
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::input_group::{KeyboardInputGroup, PadsAndKnobsInputGroup};
use std::sync::{Arc, Mutex};

pub fn sync_gui_with_data_received_from_backend(
    rx_data: &Receiver<AkaiData>,
    akai_visualizer: &Arc<Mutex<GuiData>>,
) {
    loop {
        if let Ok(x) = rx_data.recv() {
            if let Ok(mut visualizer) = akai_visualizer.lock() {
                debug!("{x:?}");
                visualizer.local_elapsed = x
                    .current_playlist
                    .as_ref()
                    .and_then(super::super::states::playlist_data::PlaylistData::get_current_track)
                    .map_or(0, |t| t.elapsed_seconds * 1000);
                visualizer.mute_on = x.button_states.contains(ToggleStates::MUTE);
                visualizer.shuffle_on = x.button_states.contains(ToggleStates::SEND);
                visualizer.loop_on = x.button_states.contains(ToggleStates::SELECT);
                visualizer.pause_on = x.button_states.contains(ToggleStates::CLIP_STOP);
                visualizer.data = x;
            } else {
                warn!("Couldn't update GUI data. GUI will not update.");
            }
        } else {
            warn!("Failed to receive data from backend. GUI will not update.");
        }
    }
}

pub fn handle_gui_command_and_relay_them_to_backend(
    rx_command: &Receiver<Command>,
    tx_command: &Sender<Command>,
    tx_data: &Sender<AkaiData>,
    music_state: &MusicState,
    sound_state: &SoundState,
    midi_out_channel: Option<MidiOutputChannels>,
) {
    loop {
        if let Ok(command) = rx_command.try_recv() {
            debug!("{command:?}");
            let mut out_channel = midi_out_channel
                .as_ref()
                .and_then(|out| out.midi_out.lock().ok());
            match command {
                Command::Refresh { device } => match device {
                    Device::ToBackend => {
                        if let Ok(audio_sink) = music_state.audio_sinks.lock()
                            && let Ok(data) = music_state.data.lock()
                        {
                            PadHandler::refresh(&data, tx_data, &audio_sink);
                        }
                    }
                    Device::ToGui => (),
                },
                Command::PadPressed { key, device } => match device {
                    Device::ToBackend => {
                        if let Ok(padkey) = PadKey::try_from(key) {
                            PadHandler::handle_input_pressed(
                                out_channel.as_deref_mut(),
                                PadsAndKnobsInputGroup::Pads(padkey),
                                1,
                                music_state,
                            );
                            refresh_backend(tx_command)
                        } else {
                            warn!("Invalid padkey, will not update data");
                        }
                    }
                    Device::ToGui => {}
                },
                Command::BlackKeyPressed { key, device }
                | Command::WhiteKeyPressed { key, device } => match device {
                    Device::ToBackend => {
                        KeyboardHandler::handle_input(KeyboardInputGroup::Key(key), &sound_state)
                    }
                    Device::ToGui => {}
                },
                Command::KnobPercentageChanged {
                    knob,
                    value,
                    device,
                } => match device {
                    Device::ToBackend => {
                        PadHandler::handle_input_pressed(
                            out_channel.as_deref_mut(),
                            PadsAndKnobsInputGroup::Knob(knob),
                            value.into(),
                            music_state,
                        );
                        refresh_backend(tx_command)
                    }
                    Device::ToGui => {}
                },
                Command::ShufflePressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::KnobCtrl(KnobCtrlKey::Send),
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                Command::LoopPressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::SoftKeys(SoftKey::Select),
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                Command::SkipTrackPressed { device } => match device {
                    Device::ToBackend => {
                        PadHandler::handle_input_pressed(
                            out_channel.as_deref_mut(),
                            PadsAndKnobsInputGroup::Right,
                            1,
                            music_state,
                        );
                        PadHandler::handle_input_released(
                            out_channel.as_deref_mut(),
                            PadsAndKnobsInputGroup::Right,
                            1,
                            music_state,
                        );
                    }
                    Device::ToGui => {}
                },
                Command::MutePressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::SoftKeys(SoftKey::Mute),
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                Command::PausePressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::SoftKeys(SoftKey::ClipStop),
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                Command::StopAllPressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::StopAllClips,
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                Command::SoloPressed { device } => match device {
                    Device::ToBackend => PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::SoftKeys(SoftKey::Solo),
                        1,
                        music_state,
                    ),
                    Device::ToGui => {}
                },
                _ => warn!("Unsupported command: {command:?}"),
            }
        }
    }
}

fn refresh_backend(tx_command: &Sender<Command>) {
    if let Err(e) = tx_command.send(Command::Refresh {
        device: Device::ToBackend,
    }) {
        warn!("Couldn't send refresh command. Error: {e}");
    }
}
