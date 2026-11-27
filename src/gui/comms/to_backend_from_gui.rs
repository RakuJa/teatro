use crate::MidiOutputChannels;
use crate::backend::extra_midi_handler::{ExtraMidiGroup, ExtraMidiHandler};
use crate::backend::hw_handler::MidiHandler;
use crate::backend::keyboard_handler::KeyboardHandler;
use crate::backend::pad_handler::PadHandler;
use crate::gui::comms::command::CommsCommand;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::RuntimeData;
use flume::{Receiver, Sender};
use ramidier::enums::button::knob_ctrl::KnobCtrlKey;
use ramidier::enums::button::pads::PadKey;
use ramidier::enums::button::soft_keys::SoftKey;
use ramidier::enums::input_group::{KeyboardInputGroup, PadsAndKnobsInputGroup};
use ramidier::io::output::ChannelOutput;
use std::sync::MutexGuard;
use tracing::{debug, warn};

pub fn handle_gui_command_and_relay_them_to_backend(
    rx_command: &Receiver<CommsCommand>,
    tx_command: &Sender<CommsCommand>,
    tx_data: &Sender<RuntimeData>,
    music_state: &MusicState,
    sound_state: &SoundState,
    midi_out_channel: Option<&MidiOutputChannels>,
) {
    while let Ok(command) = rx_command.recv() {
        debug!("{command:?}");
        match command {
            CommsCommand::Refresh => {
                if let Ok(audio_sink) = music_state.audio_sinks.lock()
                    && let Ok(data) = music_state.data.lock()
                {
                    PadHandler::refresh(&data, tx_data, &audio_sink);
                }
            }
            CommsCommand::PadPressed { key } => {
                if let Ok(padkey) = PadKey::try_from(key) {
                    press(
                        midi_out_channel,
                        PadsAndKnobsInputGroup::Pads(padkey),
                        1,
                        music_state,
                    );
                    refresh_backend(tx_command);
                } else {
                    warn!("Invalid padkey, will not update data");
                }
            }
            CommsCommand::BlackKeyPressed { key } | CommsCommand::WhiteKeyPressed { key } => {
                KeyboardHandler::handle_input(KeyboardInputGroup::Key(key), sound_state);
            }
            CommsCommand::KnobPercentageChanged { knob, value } => {
                press(
                    midi_out_channel,
                    PadsAndKnobsInputGroup::Knob(knob),
                    value.into(),
                    music_state,
                );
                refresh_backend(tx_command);
            }
            CommsCommand::ShufflePressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::KnobCtrl(KnobCtrlKey::Send),
                1,
                music_state,
            ),
            CommsCommand::LoopPressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::SoftKeys(SoftKey::Select),
                1,
                music_state,
            ),
            CommsCommand::SkipTrackPressed => {
                let mut out_channel = lock_midi_out(midi_out_channel);
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
            CommsCommand::MutePressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::SoftKeys(SoftKey::Mute),
                1,
                music_state,
            ),
            CommsCommand::PausePressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::SoftKeys(SoftKey::ClipStop),
                1,
                music_state,
            ),
            CommsCommand::StopAllPressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::StopAllClips,
                1,
                music_state,
            ),
            CommsCommand::SoloPressed => press(
                midi_out_channel,
                PadsAndKnobsInputGroup::SoftKeys(SoftKey::Solo),
                1,
                music_state,
            ),
            CommsCommand::PlayTrackPressed { index } => {
                ExtraMidiHandler::handle_input(ExtraMidiGroup::SkipToIndex(index), music_state);
                refresh_backend(tx_command);
            }
            CommsCommand::SeekTo { target } => {
                ExtraMidiHandler::handle_input(ExtraMidiGroup::SeekTo(target), music_state);
                refresh_backend(tx_command);
            }
        }
    }
    warn!("GUI command channel closed. Backend will not receive GUI commands anymore.");
}

fn lock_midi_out(
    midi_out_channel: Option<&MidiOutputChannels>,
) -> Option<MutexGuard<'_, ChannelOutput>> {
    midi_out_channel.and_then(|out| out.midi_out.lock().ok())
}

fn press(
    midi_out_channel: Option<&MidiOutputChannels>,
    input_group: PadsAndKnobsInputGroup,
    value: u8,
    music_state: &MusicState,
) {
    PadHandler::handle_input_pressed(
        lock_midi_out(midi_out_channel).as_deref_mut(),
        input_group,
        value,
        music_state,
    );
}

fn refresh_backend(tx_command: &Sender<CommsCommand>) {
    if let Err(e) = tx_command.send(CommsCommand::Refresh {}) {
        warn!("Couldn't send refresh command. Error: {e}");
    }
}
