mod audio;
mod backend;
#[cfg(feature = "gui")]
mod gui;
mod os_explorer;
mod states;

use crate::backend::listener_initializer::{prepare_midi_channels, run};
use crate::backend::pad_handler::PadHandler;
use crate::gui::initializer::gui_initializer;
use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::music_state::MusicState;
use crate::states::settings_data::SettingsData;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::RuntimeData;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use dotenvy::dotenv;
use flume::Sender;
use gui::comms::command::CommsCommand;
use log::warn;
use ramidier::io::input::InputChannel;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::env;
use std::sync::{Arc, Mutex};

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

    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.yml".to_string());

    let settings_data = SettingsData::load_from_config(&config_path).unwrap_or_default();
    let pad_labels =
        PadHandler::get_pad_albums_list(&settings_data.music_folder).unwrap_or_else(|e| {
            warn!("Could not load pads, music path is not readable: {e}");
            vec![]
        });
    let settings = Arc::new(Mutex::new(settings_data));
    let _watchdog_settings = settings.clone();

    #[cfg(feature = "gui")]
    let watchdog_settings = settings.clone();

    let backend_data = RuntimeData::builder()
        .settings_data(settings)
        .pad_labels(pad_labels)
        .build();

    let hw_data = Arc::new(Mutex::new(backend_data.clone()));
    let (tx_data, rx_data) = flume::unbounded::<RuntimeData>();
    let (gui_command_tx, gui_command_rx) = flume::unbounded::<CommsCommand>();
    let (watchgod_tx, watchdog_rx) = flume::unbounded::<CommsCommand>();

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
            if let Ok(midi_channels) = prepare_midi_channels() {
                if let Err(err) = run(&states.0, &states.1, midi_channels.0, midi_channels.1) {
                    eprintln!("MIDI Error: {err}");
                };
            };
        } else if #[cfg(all(feature = "midi", feature = "gui"))]{
            let m_state = states.0.clone();
            let s_state = states.1.clone();
            let midi_out_channels = if let Ok((in_channels, out_channels)) = prepare_midi_channels() {
                let inner_out_channels = out_channels.clone();
                std::thread::spawn(move || {
                    if let Err(err) = run(&m_state, &s_state, in_channels, inner_out_channels) {
                        eprintln!("MIDI Error: {err}");
                    }
                });
                Some(out_channels)
            } else {
                warn!("Could not create midi channels, change ports or connect the midi keyboard");
                None
            };
        } else {
            let midi_out_channels = None;
        }
    }

    #[cfg(feature = "gui")]
    {
        use crate::gui::comms::to_backend_from_gui::handle_gui_command_and_relay_them_to_backend;
        use crate::gui::comms::watchdog_handler::handle_watchdog;

        let music_state = states.0;
        let sound_state = states.1;
        let m_state_watchdog = music_state.clone();

        let gui_tx_data = tx_data.clone();
        let watchdog_tx_data = tx_data;

        let sync_tx_command = gui_command_tx.clone();

        let gui_settings = watchdog_settings.clone();

        std::thread::spawn(move || {
            handle_gui_command_and_relay_them_to_backend(
                &gui_command_rx,
                &sync_tx_command,
                &gui_tx_data,
                &music_state,
                &sound_state,
                midi_out_channels,
            );
        });

        std::thread::spawn(move || {
            handle_watchdog(
                &watchdog_settings,
                &watchdog_rx,
                &watchdog_tx_data,
                &m_state_watchdog,
            );
        });

        gui_initializer(
            backend_data,
            gui_settings,
            gui_command_tx,
            rx_data,
            watchgod_tx,
        )
        .expect("Application did not complete run correctly");
    }
}

fn get_base_filter_data(coeffs: Coefficients<f32>) -> FilterData {
    FilterData {
        previous_filter_percentage: 1.,
        filter_type: Type::AllPass,
        filter: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
    }
}

fn prepare_audio_states(
    music_queue: Sink,
    ambience_queue: Sink,
    sound_effect_queue: Sink,
    data: Arc<Mutex<RuntimeData>>,
    tx_data: &Sender<RuntimeData>,
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
    let music_filter_data = Arc::new(Mutex::new(get_base_filter_data(coeffs)));
    let ambience_filter = Arc::new(Mutex::new(get_base_filter_data(coeffs)));
    let sound_effect_filter = Arc::new(Mutex::new(get_base_filter_data(coeffs)));

    (
        MusicState {
            audio_sinks: audio_sinks.clone(),
            music_filter: music_filter_data,
            ambience_filter: ambience_filter.clone(),
            tx_data: tx_data.clone(),
            data: data.clone(),
            sound_effect_filter: sound_effect_filter.clone(),
        },
        SoundState {
            data,
            audio_sinks,
            ambience_filter,
            sound_effect_filter,
            tx_data: tx_data.clone(),
        },
    )
}
