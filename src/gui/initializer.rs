use crate::gui::comms::command::CommsCommand;
use crate::gui::comms::to_gui_from_backend::sync_gui_with_data_received_from_backend;
use crate::gui::gui_wrapper::GuiWrapper;
use crate::gui::ui::{AkaiVisualizer, GuiData};
use crate::states::settings_data::SettingsData;
use crate::states::visualizer::RuntimeData;
use flume::{Receiver, Sender};
use std::env;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub fn gui_initializer(
    backend_data: RuntimeData,
    settings: Arc<Mutex<SettingsData>>,
    tx_command: Sender<CommsCommand>,
    rx_data: Receiver<RuntimeData>,
    watchdog_tx: Sender<CommsCommand>,
) -> eframe::Result {
    let font_folder = env::var("FONT_FOLDER").unwrap_or_else(|_| "ui/fonts".to_string());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([1200.0, 800.0])
            .with_resizable(true),
        ..Default::default()
    };

    let n_of_i = env::var("INITIAL_N_OF_INFO")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<usize>()
        .expect("INITIAL_N_OF_INFO should be a positive number");
    let gui_data = GuiData::new(backend_data, tx_command, watchdog_tx);

    let arc_gui_data = Arc::new(Mutex::new(gui_data));
    let gui_data_sync = arc_gui_data.clone();
    eframe::run_native(
        "Teatro - Akai APC Key 25 Controller",
        options,
        Box::new(move |cc| {
            let ctx = &cc.egui_ctx;
            ctx.style_mut(|style| {
                style.animation_time = 0.08;
            });
            ctx.options_mut(|o| {
                if let Some(x) = std::num::NonZeroUsize::new(2) {
                    o.max_passes = x;
                }
            });
            let state = Rc::new(Mutex::new(AkaiVisualizer::new(
                cc,
                &settings,
                arc_gui_data,
                &font_folder,
                n_of_i,
            )));
            std::thread::spawn(move || {
                sync_gui_with_data_received_from_backend(&rx_data, &gui_data_sync);
            });
            Ok(Box::new(GuiWrapper { state }))
        }),
    )
}
