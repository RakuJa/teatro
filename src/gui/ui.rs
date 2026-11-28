use crate::gui::comms::command::CommsCommand;
use crate::gui::local_view::audio_player_states::PlayerInfo;
use crate::gui::navigation::CurrentTab;
use crate::gui::settings_view::settings::SettingsViewState;
use crate::states::information_data::InformationEntry;
use crate::states::settings_data::SettingsData;
use crate::states::visualizer::RuntimeData;
use eframe::egui;
use egui::Ui;
use egui_font_loader::{LoaderFontData, load_fonts};
use egui_tracing::EventCollector;
use flume::Sender;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::warn;

pub struct GuiData {
    pub(crate) data: RuntimeData,
    pub(crate) tx_to_backend: Sender<CommsCommand>,
    pub(crate) tx_to_watchdog: Sender<CommsCommand>,
    pub(crate) player_info: PlayerInfo,
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
            player_info: PlayerInfo::default(),
        }
    }
}

pub struct AkaiVisualizer {
    pub(crate) gui_data: Arc<Mutex<GuiData>>,
    pub(crate) info_panel_data: InfoPanelData,
    pub(crate) settings_data: SettingsData,
    pub(crate) settings_view: SettingsViewState,
    pub(crate) current_tab: CurrentTab,
    pub(crate) webview_error: Option<String>,
    pub(crate) event_collector: EventCollector,
}

pub struct InfoPanelData {
    pub(crate) data_file_path: String,
    pub(crate) information_list: Vec<String>,
    pub(crate) editing_index: Option<usize>,
    pub(crate) new_entry: String,
    pub(crate) focus_editor: bool,
}

impl AkaiVisualizer {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        settings_data: &Arc<Mutex<SettingsData>>,
        gui_data: Arc<Mutex<GuiData>>,
        font_folder: &str,
        data_path: &str,
        event_collector: EventCollector,
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
        let settings = settings_data
            .lock()
            .map_or_else(|_| SettingsData::default(), |s| s.clone());

        Self {
            gui_data,
            info_panel_data: InfoPanelData {
                data_file_path: data_path.to_string(),
                information_list: InformationEntry::load_from_file(data_path)
                    .unwrap_or_default()
                    .iter()
                    .map(|x| x.data.clone())
                    .collect(),
                editing_index: None,
                new_entry: String::new(),
                focus_editor: false,
            },
            settings_view: SettingsViewState::new(settings.clone()),
            settings_data: settings,
            current_tab: CurrentTab::Visualizer,
            webview_error: None,
            event_collector,
        }
    }
}

impl eframe::App for AkaiVisualizer {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::profile_function!();
        let now = Instant::now();
        let need_refresh = self.gui_data.lock().is_ok_and(|mut gui_data| {
            if now.duration_since(gui_data.player_info.last_refresh)
                >= gui_data.player_info.refresh_interval
            {
                gui_data.player_info.last_refresh = now;
                true
            } else {
                false
            }
        });

        if need_refresh {
            self.send_command_to_backend(CommsCommand::Refresh {});
        }

        self.handle_tab_shortcuts(ui);
        self.show_header(ui);
        self.show_status_bar(ui);

        egui::CentralPanel::default().show(ui, |ui| match self.current_tab {
            CurrentTab::Visualizer => self.render_visualizer_tab(ui),
            #[cfg(feature = "bybe")]
            CurrentTab::WebView => self.render_webview_tab(ui),
            CurrentTab::Settings => self.render_settings_tab(ui),
            CurrentTab::LogViewer => self.render_logviewer_tab(ui),
        });
    }
}

impl AkaiVisualizer {
    fn render_logviewer_tab(&self, ui: &mut Ui) {
        #[cfg(feature = "profiling")]
        puffin_egui::puffin::profile_function!();
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add(
                egui_tracing::Logs::new(self.event_collector.clone()), //.with_labels(self.labels.clone()),
            );
        });
    }

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
