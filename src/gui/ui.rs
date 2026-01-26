use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player_states::PlayerStates;
use crate::states::settings_data::SettingsData;
use crate::states::visualizer::RuntimeData;
use eframe::egui;
use egui_font_loader::{LoaderFontData, load_fonts};
use flume::Sender;
use log::warn;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CurrentTab {
    Visualizer,
    WebView,
    Settings,
}

pub struct GuiData {
    pub(crate) data: RuntimeData,
    pub(crate) tx_to_backend: Sender<CommsCommand>,
    pub(crate) tx_to_watchdog: Sender<CommsCommand>,
    pub(crate) audio_player_states: PlayerStates,
}

impl GuiData {
    pub fn new(
        data: RuntimeData,
        tx_to_backend: Sender<CommsCommand>,
        tx_to_watchdog: Sender<CommsCommand>,
    ) -> Self {
        Self {
            data,
            tx_to_backend,
            tx_to_watchdog,
            audio_player_states: PlayerStates::default(),
        }
    }
}

pub struct AkaiVisualizer {
    pub(crate) gui_data: Arc<Mutex<GuiData>>,
    pub(crate) info_panel_data: InfoPanelData,
    pub(crate) settings_data: SettingsData,
    pub(crate) current_tab: CurrentTab,
    pub(crate) webview_error: Option<String>,
}

pub struct InfoPanelData {
    pub(crate) combattant_lists: Vec<String>,
    pub(crate) initial_n_of_info: usize,
    pub(crate) editing_index: Option<usize>,
}

impl AkaiVisualizer {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings_data: &Arc<Mutex<SettingsData>>,
        gui_data: Arc<Mutex<GuiData>>,
        font_folder: &str,
        initial_n_of_info: usize,
    ) -> Self {
        let fonts = vec![
            LoaderFontData {
                name: "GoodTimesRg".into(),
                path: format!("{font_folder}/Good-times-rg.ttf"),
            },
            LoaderFontData {
                name: "Pixelify".into(),
                path: format!("{font_folder}/PixelifySans-VariableFont_wght.ttf"),
            },
        ];
        load_fonts(&cc.egui_ctx, fonts).expect("Font should be readable.");
        Self {
            gui_data,
            info_panel_data: InfoPanelData {
                combattant_lists: (0..initial_n_of_info)
                    .map(|i| format!("Combattant {i} | Initiative: 0"))
                    .collect(),
                initial_n_of_info,
                editing_index: None,
            },
            settings_data: settings_data
                .lock()
                .map_or_else(|_| SettingsData::default(), |s| s.clone()),
            current_tab: CurrentTab::Visualizer,
            webview_error: None,
        }
    }
}

impl eframe::App for AkaiVisualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));
        let now = Instant::now();
        let delta_time = self.gui_data.lock().map_or(0, |gui_data| {
            now.duration_since(gui_data.audio_player_states.last_local_update)
                .as_millis() as u64
        });

        if let Ok(mut gui_data) = self.gui_data.lock() {
            gui_data.audio_player_states.last_local_update = now;
        }

        let need_refresh = self.gui_data.lock().is_ok_and(|mut gui_data| {
            if now.duration_since(gui_data.audio_player_states.last_refresh)
                >= gui_data.audio_player_states.refresh_interval
            {
                gui_data.audio_player_states.last_refresh = now;
                true
            } else {
                false
            }
        });

        if need_refresh {
            self.send_command_to_backend(CommsCommand::Refresh {});
        }
        self.update_local_progress(delta_time);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, CurrentTab::Visualizer, "Teatro core");
                #[cfg(feature = "bybe")]
                {
                    ui.selectable_value(
                        &mut self.current_tab,
                        CurrentTab::WebView,
                        "web BYBE - Shop & Encounters",
                    );
                };
                ui.selectable_value(&mut self.current_tab, CurrentTab::Settings, "Settings");
            });
            ui.separator();

            match self.current_tab {
                CurrentTab::Visualizer => self.render_visualizer_tab(ui),
                CurrentTab::WebView => {
                    #[cfg(feature = "bybe")]
                    {
                        self.render_webview_tab(ui);
                    }
                }
                CurrentTab::Settings => self.render_settings_tab(ui),
            }
        });
    }
}

impl AkaiVisualizer {
    pub fn send_command_to_backend(&self, command: CommsCommand) {
        if let Ok(gui_data) = self.gui_data.lock() {
            Self::send_command(&gui_data.tx_to_backend, command);
        } else {
            warn!("Failed to lock the gui_data lock.");
        }
    }

    fn send_command(tx: &Sender<CommsCommand>, command: CommsCommand) {
        if let Err(e) = tx.send(command) {
            warn!("Failed to send {command:?} command: {e}");
        }
    }

    pub fn send_command_to_watchdog(&self, command: CommsCommand) {
        if let Ok(gui_data) = self.gui_data.lock() {
            Self::send_command(&gui_data.tx_to_watchdog, command);
        } else {
            warn!("Failed to lock the gui_data lock.");
        }
    }
}
