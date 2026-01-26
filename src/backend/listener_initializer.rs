use crate::backend::hw_handler::MidiHandler;
use crate::backend::keyboard_handler::KeyboardHandler;
use crate::backend::pad_handler::PadHandler;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::{MidiInputChannels, MidiOutputChannels};
use log::debug;
use ramidier::enums::input_group::{KeyboardChannel, PadsAndKnobsChannel};
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::enums::message_filter::MessageFilter;
use ramidier::io::input::InputChannel;
use ramidier::io::output::ChannelOutput;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn prepare_midi_channels() -> Result<(MidiInputChannels, MidiOutputChannels), Box<dyn Error>> {
    let midi_in_keyboard = InputChannel::builder()
        .port(1)
        .msg_to_ignore(MessageFilter::None)
        .build()?;

    // Setup MIDI Input
    let midi_in_pad = InputChannel::builder()
        .port(2)
        .msg_to_ignore(MessageFilter::None)
        .build()?;

    // Setup MIDI Output
    let mut midi_out = ChannelOutput::builder()
        .port(2)
        .initialize_note_led(true)
        .build()?;
    midi_out.set_all_pads_color(LedMode::On100Percent, LedColor::Off)?;

    let keyboard_midi_out = ChannelOutput::builder()
        .port(2)
        .initialize_note_led(true)
        .build()?;

    Ok((
        MidiInputChannels {
            midi_in_keyboard,
            midi_in_pad,
        },
        MidiOutputChannels {
            midi_out: Arc::new(Mutex::new(midi_out)),
            keyboard_midi_out: Arc::new(Mutex::new(keyboard_midi_out)),
        },
    ))
}

pub fn run(
    music_state: &MusicState,
    sound_state: &SoundState,
    in_channels: MidiInputChannels,
    out_channels: MidiOutputChannels,
) -> Result<(), Box<dyn Error>> {
    let _conn_in = in_channels.midi_in_pad.listen(
        Some("midir-read-input"),
        move |stamp, rx_data, data| {
            PadHandler::listener(out_channels.midi_out.clone(), stamp, &rx_data, data);
        },
        music_state.clone(),
        PadsAndKnobsChannel,
    )?;
    let _conn_keyboard = in_channels.midi_in_keyboard.listen(
        Some("midir-keyboard-read-input"),
        move |stamp, rx_data, data| {
            KeyboardHandler::listener(
                out_channels.keyboard_midi_out.clone(),
                stamp,
                &rx_data,
                data,
            );
        },
        sound_state.clone(),
        KeyboardChannel,
    )?;
    // Just keep the program alive - MIDI callbacks will handle input
    debug!("MIDI listeners active");
    loop {
        thread::park();
    }
}
