use crate::gui::ui::AkaiVisualizer;
use egui::{Color32, RichText};
use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

// Global flag to track if a webview window is currently open
static WEBVIEW_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

impl AkaiVisualizer {
    pub(crate) fn create_webview_window() {
        // Check if a window is already open
        if WEBVIEW_WINDOW_OPEN.load(Ordering::Relaxed) {
            return;
        }

        // Mark window as open
        WEBVIEW_WINDOW_OPEN.store(true, Ordering::Relaxed);

        // Spawn a separate thread for the webview window
        thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let event_loop = {
                use tao::platform::unix::EventLoopBuilderExtUnix;
                EventLoopBuilder::new().with_any_thread(true).build()
            };

            #[cfg(not(target_os = "linux"))]
            let event_loop = EventLoopBuilder::new().build();

            if let Ok(window) = WindowBuilder::new()
                .with_title("Teatro - Web Browser (Close main app to exit)")
                .with_inner_size(tao::dpi::LogicalSize::new(1200, 800))
                .build(&event_loop)
            {
                let builder = WebViewBuilder::new()
                    .with_url("https://bybe.fly.dev/")
                    .with_new_window_req_handler(|url, features| {
                        debug!("new window req: {url} {features:?}");
                        wry::NewWindowResponse::Allow
                    });

                #[cfg(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "android"
                ))]
                let _webview = builder.build(&window);

                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "ios",
                    target_os = "android"
                )))]
                let _webview = {
                    use tao::platform::unix::WindowExtUnix;
                    use wry::WebViewBuilderExtUnix;
                    let vbox = window.default_vbox().expect("no default vbox");
                    builder.build_gtk(vbox)
                };

                info!("Separate webview window created!");
                info!("NOTE: This window cannot be closed independently.");
                info!("Please close the main application window to exit.");

                event_loop.run(move |event, _, control_flow| {
                    *control_flow = ControlFlow::Wait;

                    if let Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } = event
                    {
                        warn!(
                            "Close request ignored. Please close the main application window instead."
                        );
                    }
                });
            }
        });
    }

    pub(crate) fn render_webview_tab(&self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading(
                RichText::new("Web Browser")
                    .color(Color32::from_rgb(100, 150, 255))
                    .size(20.0),
            );
            ui.add_space(5.0);
        });

        ui.separator();

        // Show error message if webview creation failed
        if let Some(error) = &self.webview_error.clone() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label(
                    RichText::new("⚠ WebView Error")
                        .color(Color32::from_rgb(255, 100, 100))
                        .size(18.0),
                );
                ui.add_space(10.0);
                ui.label(RichText::new(error).color(Color32::LIGHT_RED));
                ui.add_space(20.0);

                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Solutions:");
                        ui.add_space(10.0);

                        if std::env::var("WAYLAND_DISPLAY").is_ok()
                            && std::env::var("GDK_BACKEND").unwrap_or_default() != "x11"
                        {
                            ui.label("You're running on Wayland. Restart with:");
                            ui.code("GDK_BACKEND=x11 cargo run");
                            ui.add_space(5.0);
                            ui.label("OR");
                            ui.add_space(5.0);
                        }

                        if ui
                            .button("🔗 Open Web Browser in Separate Window")
                            .clicked()
                        {
                            Self::create_webview_window();
                        }
                    });
                });
            });
            return;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(50.0);

            let window_is_open = WEBVIEW_WINDOW_OPEN.load(Ordering::Relaxed);

            if window_is_open {
                ui.label(
                    RichText::new("Web Browser Window is Open")
                        .size(16.0)
                        .color(Color32::from_rgb(100, 200, 100)),
                );
                ui.add_space(20.0);
                ui.label("The web browser is running in a separate window.");
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("⚠ Important")
                                .size(14.0)
                                .color(Color32::from_rgb(255, 200, 100)),
                        );
                        ui.add_space(5.0);
                        ui.label("The web browser window cannot be closed independently.");
                        ui.label("To exit, close the main Teatro application window.");
                    });
                });
            } else {
                ui.label(
                    RichText::new("Web Browser - Separate Window Mode")
                        .size(16.0)
                        .color(Color32::from_rgb(150, 150, 150)),
                );
                ui.add_space(20.0);
                ui.label("Click the button below to open the web browser");
                ui.label("in a separate, independent window.");
                ui.add_space(20.0);

                if ui
                    .button(RichText::new("Open Web Browser Window").size(16.0))
                    .clicked()
                {
                    Self::create_webview_window();
                }

                ui.add_space(20.0);
                ui.group(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("ℹ Note")
                                .size(12.0)
                                .color(Color32::from_rgb(150, 150, 150)),
                        );
                        ui.add_space(5.0);
                        ui.label("Once opened, the browser window will stay open");
                        ui.label("until you close the main application.");
                    });
                });
            }
        });
    }
}
