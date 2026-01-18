mod audio;
#[cfg(feature = "gui")]
mod gui;
mod hardware_handler;
mod os_explorer;
mod states;

use crate::hardware_handler::keyboard_handler::keyboard_listener_logic;
use crate::hardware_handler::pad_handler::pad_listener_logic;
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

fn run(tx_data: &Sender<AkaiData>, data: AkaiData) -> Result<(), Box<dyn Error>> {
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

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let music_queue = Sink::connect_new(stream_handle.mixer());
    let ambience_queue = Sink::connect_new(stream_handle.mixer());
    let sound_effect_queue = Sink::connect_new(stream_handle.mixer());

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
    let _conn_in = midi_in_pad.listen(
        Some("midir-read-input"),
        move |stamp, rx_data, data| pad_listener_logic(&mut midi_out, stamp, &rx_data, data),
        MusicState {
            audio_sinks: audio_sinks.clone(),
            music_filter: music_filter_data,
            sound_filter: sound_filter_data.clone(),
            tx_data: tx_data.clone(),
            data: data.clone(),
        },
        PadsAndKnobsChannel,
    )?;
    let _conn_keyboard = midi_in_keyboard.listen(
        Some("midir-keyboard-read-input"),
        move |stamp, rx_data, data| keyboard_listener_logic(stamp, &rx_data, data),
        SoundState {
            data,
            audio_sinks,
            sound_filter: sound_filter_data,
            tx_data: tx_data.clone(),
        },
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

    let gui_data = AkaiData::builder()
        .music_folder(&music_folder)
        .sound_folder(&sound_folder)
        .build();
    let hw_data = gui_data.clone();
    let (tx_data, rx_data) = flume::unbounded::<AkaiData>();

    #[cfg(feature = "gui")]
    {
        std::thread::spawn(move || {
            if let Err(err) = run(&tx_data, hw_data) {
                eprintln!("MIDI Error: {err}");
            }
        });
    }
    #[cfg(not(feature = "gui"))]
    {
        if let Err(err) = crate::run(&tx_data, hw_data) {
            eprintln!("MIDI Error: {err}");
        }
    }
    #[cfg(feature = "gui")]
    {
        use crate::gui::gui_wrapper::GuiWrapper;
        use crate::gui::handler::AkaiVisualizer;
        use crate::gui::sync_handler::sync_gui_with_hardware;
        let state = Arc::new(Mutex::new(AkaiVisualizer { data: gui_data }));

        let wrapper = GuiWrapper {
            state: state.clone(),
        };

        std::thread::spawn(move || sync_gui_with_hardware(&rx_data, &state));
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1400.0, 800.0])
                .with_min_inner_size([1000.0, 600.0])
                .with_resizable(true),
            ..Default::default()
        };

        eframe::run_native(
            "Teatro - Akai APC Key 25 Controller",
            options,
            Box::new(move |_cc| Ok(Box::new(wrapper))),
        )
        .expect("Run did not complete run correctly");
    }
}
