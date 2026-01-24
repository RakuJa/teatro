use crate::comms::command::{Command, Device};
use crate::states::visualizer::AkaiData;
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
}

pub struct GuiData {
    pub(crate) data: AkaiData,
    pub(crate) tx_command: Sender<Command>,
    last_refresh: Instant,
    last_local_update: Instant,
    refresh_interval: Duration,
    pub(crate) local_elapsed: u64,
    pub(crate) shuffle_on: bool,
    pub(crate) loop_on: bool,
    pub(crate) mute_on: bool,
    pub(crate) pause_on: bool,
    pub(crate) solo_on: bool,
    pub(crate) stop_all_on: bool,
}

impl GuiData {
    pub fn new(data: AkaiData, tx_command: Sender<Command>) -> Self {
        Self {
            data,
            tx_command,
            last_refresh: Instant::now(),
            last_local_update: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            local_elapsed: 0,
            shuffle_on: false,
            loop_on: false,
            mute_on: false,
            pause_on: false,
            solo_on: false,
            stop_all_on: false,
        }
    }
}

pub struct AkaiVisualizer {
    pub(crate) gui_data: Arc<Mutex<GuiData>>,
    pub(crate) info_panel_data: InfoPanelData,
    pub(crate) current_tab: CurrentTab,
    #[cfg(feature = "bybe")]
    pub(crate) webview: Option<Arc<Mutex<wry::WebView>>>,
    #[cfg(not(feature = "bybe"))]
    pub(crate) webview: Option<()>,
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
            current_tab: CurrentTab::Visualizer,
            webview: None,
            webview_error: None,
        }
    }
}

impl eframe::App for AkaiVisualizer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(16));
        let now = Instant::now();
        let delta_time = self.gui_data.lock().map_or(0, |gui_data| {
            now.duration_since(gui_data.last_local_update).as_millis() as u64
        });

        if let Ok(mut gui_data) = self.gui_data.lock() {
            gui_data.last_local_update = now;
        }

        let need_refresh = self.gui_data.lock().is_ok_and(|mut gui_data| {
            if now.duration_since(gui_data.last_refresh) >= gui_data.refresh_interval {
                gui_data.last_refresh = now;
                true
            } else {
                false
            }
        });

        if need_refresh {
            self.send_command(Command::Refresh {
                device: Device::ToBackend,
            });
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
                }
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
            }
        });
    }
}

impl AkaiVisualizer {
    pub fn send_command(&self, command: Command) {
        if let Ok(gui_data) = self.gui_data.lock() {
            if let Err(e) = gui_data.tx_command.send(command) {
                warn!("Failed to send {command:?} command: {e}");
            }
        } else {
            warn!("Failed to lock the gui_data lock.");
        }
    }
}
