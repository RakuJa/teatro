use crate::MidiOutputChannels;
use crate::backend::hw_handler::MidiHandler;
use crate::backend::keyboard_handler::KeyboardHandler;
use crate::backend::pad_handler::PadHandler;
use crate::gui::comms::command::Command;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::RuntimeData;
use flume::{Receiver, Sender};
use log::{debug, warn};
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::input_group::{KeyboardInputGroup, PadsAndKnobsInputGroup};

pub fn handle_gui_command_and_relay_them_to_backend(
    rx_command: &Receiver<Command>,
    tx_command: &Sender<Command>,
    tx_data: &Sender<RuntimeData>,
    music_state: &MusicState,
    sound_state: &SoundState,
    midi_out_channel: Option<MidiOutputChannels>,
) {
    loop {
        if let Ok(command) = rx_command.recv() {
            debug!("{command:?}");
            let mut out_channel = midi_out_channel
                .as_ref()
                .and_then(|out| out.midi_out.lock().ok());
            match command {
                Command::Refresh => {
                    if let Ok(audio_sink) = music_state.audio_sinks.lock()
                        && let Ok(data) = music_state.data.lock()
                    {
                        PadHandler::refresh(&data, tx_data, &audio_sink);
                    }
                }
                Command::PadPressed { key } => {
                    if let Ok(padkey) = PadKey::try_from(key) {
                        PadHandler::handle_input_pressed(
                            out_channel.as_deref_mut(),
                            PadsAndKnobsInputGroup::Pads(padkey),
                            1,
                            music_state,
                        );
                        refresh_backend(tx_command);
                    } else {
                        warn!("Invalid padkey, will not update data");
                    }
                }
                Command::BlackKeyPressed { key } | Command::WhiteKeyPressed { key } => {
                    KeyboardHandler::handle_input(KeyboardInputGroup::Key(key), sound_state);
                }
                Command::KnobPercentageChanged { knob, value } => {
                    PadHandler::handle_input_pressed(
                        out_channel.as_deref_mut(),
                        PadsAndKnobsInputGroup::Knob(knob),
                        value.into(),
                        music_state,
                    );
                    refresh_backend(tx_command);
                }
                Command::ShufflePressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::KnobCtrl(KnobCtrlKey::Send),
                    1,
                    music_state,
                ),
                Command::LoopPressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::SoftKeys(SoftKey::Select),
                    1,
                    music_state,
                ),
                Command::SkipTrackPressed => {
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
                Command::MutePressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::SoftKeys(SoftKey::Mute),
                    1,
                    music_state,
                ),
                Command::PausePressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::SoftKeys(SoftKey::ClipStop),
                    1,
                    music_state,
                ),
                Command::StopAllPressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::StopAllClips,
                    1,
                    music_state,
                ),
                Command::SoloPressed => PadHandler::handle_input_pressed(
                    out_channel.as_deref_mut(),
                    PadsAndKnobsInputGroup::SoftKeys(SoftKey::Solo),
                    1,
                    music_state,
                ),
                _ => warn!("Unsupported command: {command:?}"),
            }
        }
    }
}

fn refresh_backend(tx_command: &Sender<Command>) {
    if let Err(e) = tx_command.send(Command::Refresh {}) {
        warn!("Couldn't send refresh command. Error: {e}");
    }
}
