mod server;

use eframe::egui;
use std::thread;

// Enum to represent the different tabs
#[derive(PartialEq, Eq)]
enum Tab {
    Dashboard,
    Monitoring,
    Files,
}

// Enum for protocol selection
#[derive(PartialEq, Eq, Clone, Copy, Debug)] // Added Debug for ComboBox text
enum Protocol {
    HTTP,
    HTTPS,
}

struct SolaraApp {
    server_status: String,
    active_tab: Tab, // Track the active tab
    is_server_running: bool, // Track if the server is currently running
    host: String,
    port: u16,
    protocol: Protocol,
}

impl eframe::App for SolaraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Left panel for tab navigation
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Navigation");
            ui.separator();

            // Use radio buttons for tab selection
            ui.selectable_value(&mut self.active_tab, Tab::Dashboard, "Dashboard");
            ui.selectable_value(&mut self.active_tab, Tab::Monitoring, "Monitoring");
            ui.selectable_value(&mut self.active_tab, Tab::Files, "Files");

            ui.separator();
            // Display server status at the bottom of the side panel
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                 ui.separator();
                 ui.label(&self.server_status);
            });
        });

        // Central panel for tab content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Dashboard => {
                    ui.heading("Server Dashboard");
                    ui.separator();

                    // Server Status Display
                    ui.label(format!("Status: {}", if self.is_server_running { "Running" } else { "Stopped" }));
                    ui.label(&self.server_status); // Keep the detailed status message for now
                    ui.separator();

                    // Server Control Buttons
                    ui.horizontal(|ui| {
                        // Use ui.add_enabled for conditional buttons
                        if ui.add_enabled(!self.is_server_running, egui::Button::new("Start Server")).clicked() {
                            println!("Start Server button clicked"); // Placeholder action
                            // TODO: Implement actual server start logic
                            self.is_server_running = true;
                            self.server_status = "Server starting...".to_string(); // Update status
                        }

                        if ui.add_enabled(self.is_server_running, egui::Button::new("Stop Server")).clicked() {
                            println!("Stop Server button clicked"); // Placeholder action
                            // TODO: Implement actual server stop logic
                            self.is_server_running = false;
                            self.server_status = "Server stopped.".to_string(); // Update status
                        }
                    });

                    ui.separator();
                    ui.label("Configuration:");
                    // Configuration Inputs
                    egui::Grid::new("config_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Host:");
                            // Use ui.add_enabled for TextEdit
                            ui.add_enabled(!self.is_server_running, egui::TextEdit::singleline(&mut self.host));
                            ui.end_row();

                            ui.label("Port:");
                            // Use ui.add_enabled for DragValue
                            ui.add_enabled(!self.is_server_running, egui::DragValue::new(&mut self.port).clamp_range(1..=65535));
                            ui.end_row();

                            ui.label("Protocol:");
                            // Wrap ComboBox logic in ui.add_enabled
                            ui.add_enabled_ui(!self.is_server_running, |ui| {
                                egui::ComboBox::from_label("")
                                    .selected_text(format!("{:?}", self.protocol))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.protocol, Protocol::HTTP, "HTTP");
                                        ui.selectable_value(&mut self.protocol, Protocol::HTTPS, "HTTPS");
                                    })
                                    .response.on_hover_text("Select HTTP or HTTPS"); // Apply hover text to the response inside
                            });
                            ui.end_row();
                        });

                    // Save/Load Buttons
                    ui.horizontal(|ui| {
                        // Use ui.add_enabled for conditional buttons
                        if ui.add_enabled(!self.is_server_running, egui::Button::new("Save Config")).clicked() {
                            println!("Save Config button clicked"); // Placeholder action
                            // TODO: Implement configuration saving logic
                        }

                        if ui.add_enabled(!self.is_server_running, egui::Button::new("Load Config")).clicked() {
                            println!("Load Config button clicked"); // Placeholder action
                            // TODO: Implement configuration loading logic
                        }
                    });
                }
                Tab::Monitoring => {
                    ui.heading("Server Monitoring");
                    ui.label("This is the monitoring view.");
                    // TODO: Add monitoring components here (stats, logs, etc.)
                }
                Tab::Files => {
                    ui.heading("File Management");
                    ui.label("This is the file management view.");
                    // TODO: Add file browser and management tools here
                }
            }
        });
    }
}

impl SolaraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // For now, assume server starts automatically as before
        // We'll refine this later to use the button logic
        let initial_server_running = true;
        Self {
            server_status: "Server running at http://localhost:8000".to_string(), // Update this based on config later
            active_tab: Tab::Dashboard, // Default to Dashboard tab
            is_server_running: initial_server_running,
            host: "127.0.0.1".to_string(), // Default host
            port: 8000, // Default port
            protocol: Protocol::HTTP, // Default protocol
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Start the web server in a separate thread
    thread::spawn(|| {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        runtime.block_on(async {
            server::start_server().await.expect("Failed to start server");
        });
    });

    // Start the GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Solara Server GUI",
        options,
        Box::new(|cc| Box::new(SolaraApp::new(cc))),
    )
}
