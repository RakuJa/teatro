use crate::backend::pad_handler::PadHandler;
use crate::gui::comms::command::CommsCommand;
use crate::states::music_state::MusicState;
use crate::states::settings_data::SettingsData;
use crate::states::visualizer::RuntimeData;
use flume::{Receiver, Sender};
use hotwatch::{Event, EventKind, Hotwatch};
use log::warn;
use std::sync::{Arc, Mutex};

fn update_pads(music_state: &MusicState, tx_data: &Sender<RuntimeData>) {
    if let Ok(mut data) = music_state.data.lock()
        && let Ok(new_data) = PadHandler::update_pad_albums_list(&data, tx_data)
    {
        data.copy_data(new_data);
    }
}

fn observe_folder(
    watchdog: &mut Hotwatch,
    music_folder: &str,
    music_state: MusicState,
    tx_data: Sender<RuntimeData>,
) {
    if let Err(e) = watchdog.watch(music_folder, move |event: Event| {
        if let EventKind::Modify(_) = event.kind {
            update_pads(&music_state, &tx_data);
        }
    }) {
        warn!("Error while observing new folder: {e}");
    }
}

pub fn handle_watchdog(
    settings_data: &Arc<Mutex<SettingsData>>,
    rx_command: &Receiver<CommsCommand>,
    tx_data: &Sender<RuntimeData>,
    music_state: &MusicState,
) {
    let mut hotwatch = Hotwatch::new().expect("hotwatch failed to initialize!");
    let mut last_folder = settings_data
        .lock()
        .map_or_else(|_| "music".to_string(), |x| x.music_folder.clone());
    let mut m_state_watchdog = music_state.clone();
    let mut tx_data_watchdog = tx_data.clone();

    observe_folder(
        &mut hotwatch,
        &last_folder,
        m_state_watchdog,
        tx_data_watchdog,
    );

    loop {
        if let Ok(command) = rx_command.recv() {
            if matches!(command, CommsCommand::Refresh) {
                if let Err(e) = hotwatch.unwatch(&last_folder) {
                    warn!("Error while unwatching folder: {e}");
                } else if let Ok(s) = settings_data.lock() {
                    last_folder.clone_from(&s.music_folder);
                }
                m_state_watchdog = music_state.clone();
                tx_data_watchdog = tx_data.clone();

                update_pads(music_state, tx_data);
                observe_folder(
                    &mut hotwatch,
                    &last_folder,
                    m_state_watchdog,
                    tx_data_watchdog,
                );
            }
        }
    }
}
