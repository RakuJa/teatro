mod audio;
mod comms;
#[cfg(feature = "gui")]
mod gui;
mod hardware_handler;
mod os_explorer;
mod states;

use crate::comms::command::Command;
use crate::hardware_handler::keyboard_handler::KeyboardHandler;
use crate::hardware_handler::midi_handler::MidiHandler;
use crate::hardware_handler::pad_handler::PadHandler;
use crate::states::audio_sinks::AudioSinks;
use crate::states::filter_data::FilterData;
use crate::states::music_state::MusicState;
use crate::states::sound_state::SoundState;
use crate::states::visualizer::AkaiData;
use biquad::{Coefficients, DirectForm1, Q_BUTTERWORTH_F32, ToHertz, Type};
use dotenv::dotenv;
use flume::Sender;
use ramidier::enums::input_group::{KeyboardChannel, PadsAndKnobsChannel};
use ramidier::enums::led_light::color::LedColor;
use ramidier::enums::led_light::mode::LedMode;
use ramidier::enums::message_filter::MessageFilter;
use ramidier::io::output::ChannelOutput;
use rodio::Sink;
use std::env;
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex};

fn prepare_states(
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

fn run(music_state: &MusicState, sound_state: &SoundState) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let midi_in_keyboard = ramidier::io::input::InputChannel::builder()
        .port(1)
        .msg_to_ignore(MessageFilter::None)
        .build()?;

    // Setup MIDI Input
    let midi_in_pad = ramidier::io::input::InputChannel::builder()
        .port(2)
        .msg_to_ignore(MessageFilter::None)
        .build()?;

    // Setup MIDI Output
    let mut midi_out = ChannelOutput::builder()
        .port(2)
        .initialize_note_led(true)
        .build()?;
    midi_out.set_all_pads_color(LedMode::On100Percent, LedColor::Off)?;

    let mut keyboard_midi_out = ChannelOutput::builder()
        .port(2)
        .initialize_note_led(true)
        .build()?;

    let _conn_in = midi_in_pad.listen(
        Some("midir-read-input"),
        move |stamp, rx_data, data| PadHandler::listener(&mut midi_out, stamp, &rx_data, data),
        music_state.clone(),
        PadsAndKnobsChannel,
    )?;
    let _conn_keyboard = midi_in_keyboard.listen(
        Some("midir-keyboard-read-input"),
        move |stamp, rx_data, data| {
            KeyboardHandler::listener(&mut keyboard_midi_out, stamp, &rx_data, data);
        },
        sound_state.clone(),
        KeyboardChannel,
    )?;
    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press
    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let music_folder = env::var("MUSIC_FOLDER").unwrap_or_else(|_| "music".to_string());
    let sound_folder = env::var("SOUND_FOLDER").unwrap_or_else(|_| "sound".to_string());

    let pad_labels =
        PadHandler::get_pad_albums_list(&music_folder).expect("Music folder should be readable");

    let gui_data = AkaiData::builder()
        .music_folder(&music_folder)
        .sound_folder(&sound_folder)
        .pad_labels(pad_labels)
        .build();

    println!("Main: {:?}", gui_data.knob_values);
    let hw_data = Arc::new(Mutex::new(gui_data.clone()));
    let (tx_data, rx_data) = flume::unbounded::<AkaiData>();
    let (tx_command, rx_command) = flume::unbounded::<Command>();

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()
        .expect("Audio stream should be writable and readable");
    let music_queue = Sink::connect_new(stream_handle.mixer());
    let ambience_queue = Sink::connect_new(stream_handle.mixer());
    let sound_effect_queue = Sink::connect_new(stream_handle.mixer());

    let (music_state, sound_state) = prepare_states(
        music_queue,
        ambience_queue,
        sound_effect_queue,
        hw_data,
        &tx_data,
    );

    #[cfg(not(feature = "gui"))]
    {
        if let Err(err) = run(&music_state, &sound_state) {
            eprintln!("MIDI Error: {err}");
        }
    }

    #[cfg(feature = "gui")]
    {
        use crate::gui::gui_wrapper::GuiWrapper;
        use crate::gui::sync_handler::handle_gui_command_and_relay_them_to_backend;
        use crate::gui::sync_handler::sync_gui_with_data_received_from_backend;
        use crate::gui::ui::AkaiVisualizer;
        use hotwatch::{Event, EventKind, Hotwatch};
        let m_state = music_state.clone();
        let m_state_watchdog = music_state.clone();
        let gui_tx_data = tx_data.clone();

        std::thread::spawn(move || {
            if let Err(err) = run(&m_state, &sound_state) {
                eprintln!("MIDI Error: {err}");
            }
        });
        std::thread::spawn(move || {
            handle_gui_command_and_relay_them_to_backend(&rx_command, &gui_tx_data, &music_state);
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

        let font_folder = env::var("FONT_FOLDER").unwrap_or_else(|_| "ui/fonts".to_string());

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400.0, 800.0])
                .with_min_inner_size([1000.0, 600.0])
                .with_resizable(true),
            ..Default::default()
        };

        let n_of_c = env::var("INITIAL_N_OF_COMBATTANT")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<usize>()
            .expect("N_OF_C should be a usize");

        eframe::run_native(
            "Teatro - Akai APC Key 25 Controller",
            options,
            Box::new(move |cc| {
                let state = Arc::new(Mutex::new(AkaiVisualizer::new(
                    cc,
                    gui_data,
                    tx_command,
                    &font_folder,
                    n_of_c,
                )));
                let sync_state = state.clone();
                std::thread::spawn(move || {
                    sync_gui_with_data_received_from_backend(&rx_data, &sync_state);
                });
                Ok(Box::new(GuiWrapper { state }))
            }),
        )
        .expect("Application did not complete run correctly");
    }
}
