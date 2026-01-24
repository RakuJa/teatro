mod audio;
mod comms;
#[cfg(feature = "gui")]
mod gui;
mod hardware_handler;
mod os_explorer;
mod states;

use crate::comms::command::Command;
use crate::hardware_handler::pad_handler::PadHandler;
use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::AkaiData;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use dotenv::dotenv;
use flume::{Sender};
use ramidier::io::input::InputChannel;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::env;
use std::sync::{Arc, Mutex};
use crate::gui::initializer::gui_initializer;

fn prepare_audio_states(
    music_queue: Sink,
    ambience_queue: Sink,
    sound_effect_queue: Sink,
    data: Arc<Mutex<AkaiData>>,
    tx_data: &Sender<AkaiData>,
) -> (MusicState, SoundState) {
    let audio_sinks = Arc::new(Mutex::new(AudioSinks {
        music_queue,
        ambience_queue,
        sound_effect_queue,
    }));

    let sample_rate = 44100.0;
    let coeffs = Coefficients::<f32>::from_params(
        Type::AllPass,
        sample_rate.hz(),
        44100.hz(),
        Q_BUTTERWORTH_F32,
    )
    .expect("Could not create coeffs to initialize filters");
    let music_filter_data = Arc::new(Mutex::new(FilterData {
        previous_filter_percentage: 1.,
        filter_type: Type::AllPass,
        filter: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
    }));
    let sound_filter_data = Arc::new(Mutex::new(FilterData {
        previous_filter_percentage: 1.,
        filter_type: Type::AllPass,
        filter: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
    }));

    (
        MusicState {
            audio_sinks: audio_sinks.clone(),
            music_filter: music_filter_data,
            sound_filter: sound_filter_data.clone(),
            tx_data: tx_data.clone(),
            data: data.clone(),
        },
        SoundState {
            data,
            audio_sinks,
            sound_filter: sound_filter_data,
            tx_data: tx_data.clone(),
        },
    )
}

#[derive(Clone)]
pub struct MidiOutputChannels {
    pub midi_out: Arc<Mutex<ChannelOutput>>,
    pub keyboard_midi_out: Arc<Mutex<ChannelOutput>>,
}

pub struct MidiInputChannels {
    pub midi_in_keyboard: InputChannel,
    pub midi_in_pad: InputChannel,
}

fn main() {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let music_folder = env::var("MUSIC_FOLDER").unwrap_or_else(|_| "music".to_string());
    let sound_folder = env::var("SOUND_FOLDER").unwrap_or_else(|_| "sound".to_string());

    let pad_labels =
        PadHandler::get_pad_albums_list(&music_folder).expect("Music folder should be readable");

    let backend_data = AkaiData::builder()
        .music_folder(&music_folder)
        .sound_folder(&sound_folder)
        .pad_labels(pad_labels)
        .build();

    let hw_data = Arc::new(Mutex::new(backend_data.clone()));
    let (tx_data, rx_data) = flume::unbounded::<AkaiData>();
    let (tx_command, rx_command) = flume::unbounded::<Command>();

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("Audio stream should be writable and readable");
    let music_queue = Sink::connect_new(stream_handle.mixer());
    let ambience_queue = Sink::connect_new(stream_handle.mixer());
    let sound_effect_queue = Sink::connect_new(stream_handle.mixer());

    let states = prepare_audio_states(
        music_queue,
        ambience_queue,
        sound_effect_queue,
        hw_data,
        &tx_data,
    );

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "midi", not(feature = "gui")))] {
            use crate::hardware_handler::listener_initializer::{prepare_midi_channels, run};
            if let Ok(midi_channels) = prepare_midi_channels() {
                if let Err(err) = run(&states.0, &states.1, midi_channels.0, midi_channels.1) {
                    eprintln!("MIDI Error: {err}");
                };
            };
        } else if #[cfg(all(feature = "midi", feature = "gui"))]{
            use crate::hardware_handler::listener_initializer::{prepare_midi_channels, run};
            let m_state = states.0.clone();
            let s_state = states.1.clone();
            let midi_channels = prepare_midi_channels().unwrap();
            let in_channels = midi_channels.0;
            let inner_out_channels = midi_channels.1.clone();
            let midi_out_channels = Some(midi_channels.1);
            std::thread::spawn(move || {
                if let Err(err) = run(&m_state, &s_state, in_channels, inner_out_channels) {
                    eprintln!("MIDI Error: {err}");
                }
            });
        } else {
            let midi_out_channels = None;
        }
    }

    #[cfg(feature = "gui")]
    {
        use crate::gui::sync_handler::handle_gui_command_and_relay_them_to_backend;
        use hotwatch::{Event, EventKind, Hotwatch};
        let music_state = states.0;
        let sound_state = states.1;
        let m_state_watchdog = music_state.clone();

        let gui_tx_data = tx_data.clone();
        let sync_tx_command = tx_command.clone();

        std::thread::spawn(move || {
            handle_gui_command_and_relay_them_to_backend(
                &rx_command,
                &sync_tx_command,
                &gui_tx_data,
                &music_state,
                &sound_state,
                midi_out_channels,
            );
        });

        let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
        hotwatch
            .watch(music_folder, move |event: Event| {
                if let EventKind::Modify(_) = event.kind {
                    if let Ok(mut data) = m_state_watchdog.data.lock() {
                        if let Ok(new_data) = PadHandler::update_pad_albums_list(&data, &tx_data) {
                            data.copy_data(new_data);
                        }
                    }
                }
            })
            .expect("failed to watch folder!");
        gui_initializer(backend_data, tx_command, rx_data).expect("Application did not complete run correctly");
    }
}
