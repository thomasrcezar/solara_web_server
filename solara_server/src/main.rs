mod server;

use eframe::egui;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use server::{ServerConfig, ServerStatus, ServerStats}; // Import new types

// Enum to represent the different tabs
#[derive(PartialEq, Eq, Clone, Copy)]
enum Tab {
    Dashboard,
    Monitoring,
    Files,
}

// Enum for protocol selection - Keep for UI, but server.rs doesn't use it yet
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Protocol {
    HTTP,
    HTTPS,
}

struct SolaraApp {
    server_status: ServerStatus,
    active_tab: Tab,
    host: String,
    port: u16,
    protocol: Protocol,
    tokio_handle: Handle,
    shutdown_sender: Option<oneshot::Sender<()>>,
    server_stats: ServerStats, // Add server stats
    logs: Vec<String>, // Add a field to store logs
}

impl eframe::App for SolaraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use a static variable to track when we last updated stats
        static mut LAST_UPDATE: Option<std::time::Instant> = None;
        
        // Update stats every second
        unsafe {
            let now = std::time::Instant::now();
            if LAST_UPDATE.is_none() || now.duration_since(LAST_UPDATE.unwrap()).as_secs() >= 1 {
                self.update_stats();
                LAST_UPDATE = Some(now);
            }
        }
        
        // Request repaint continuously to check server status (can be optimized later)
        ctx.request_repaint();

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
                 // Display the ServerStatus enum
                 ui.label(format!("Status: {}", self.server_status));
            });
        });

        // Central panel for tab content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Dashboard => {
                    ui.heading("Server Dashboard");
                    ui.separator();

                    // Server Status Display (already in side panel, maybe remove here?)
                    // ui.label(format!("Status: {}", self.server_status));
                    // ui.separator();

                    // Determine if server is in a startable or stoppable state
                    let can_start = matches!(self.server_status, ServerStatus::Stopped | ServerStatus::Error(_));
                    let can_stop = matches!(self.server_status, ServerStatus::Running | ServerStatus::Starting); // Allow stopping even if starting

                    // Server Control Buttons
                    ui.horizontal(|ui| {
                        if ui.add_enabled(can_start, egui::Button::new("Start Server")).clicked() {
                            println!("Start Server button clicked");
                            self.server_status = ServerStatus::Starting;

                            let (tx, rx) = oneshot::channel();
                            self.shutdown_sender = Some(tx); // Store the sender

                            let config = ServerConfig {
                                host: self.host.clone(),
                                port: self.port,
                                max_connections: 100, // Add this field with a default value
                            };
                            let handle = self.tokio_handle.clone();

                            handle.spawn(async move {
                                // Phase 1: Ignite the server (no longer needs rx)
                                match server::start_server(config.clone()).await { // Removed rx here
                                    Ok(rocket_instance) => {
                                        println!("Server ignited successfully.");
                                        // Phase 2: Launch the ignited server, passing the receiver again
                                        match server::launch_ignited_server(rocket_instance, rx).await { // Re-added rx here
                                            Ok(_) => {
                                                println!("Server shut down gracefully.");
                                            }
                                            Err(e) => {
                                                eprintln!("Server launch failed: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Server ignition failed: {}", e);
                                    }
                                }
                            });
                            self.server_status = ServerStatus::Running; // Simplification for now
                        }

                        if ui.add_enabled(can_stop, egui::Button::new("Stop Server")).clicked() {
                            println!("Stop Server button clicked");
                            if let Some(sender) = self.shutdown_sender.take() {
                                self.server_status = ServerStatus::Stopping;
                                if sender.send(()).is_ok() {
                                    println!("Shutdown signal sent.");
                                } else {
                                    eprintln!("Failed to send shutdown signal (receiver dropped?).");
                                    self.server_status = ServerStatus::Stopped; // Assume stopped
                                }
                            } else {
                                println!("Server not running or already stopping.");
                                self.server_status = ServerStatus::Stopped;
                            }
                        }
                    });

                    ui.separator();
                    ui.label("Configuration:");
                    // Configuration Inputs - Disable when server is not stopped/error
                    let config_enabled = matches!(self.server_status, ServerStatus::Stopped | ServerStatus::Error(_));
                    egui::Grid::new("config_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Host:");
                            ui.add_enabled(config_enabled, egui::TextEdit::singleline(&mut self.host));
                            ui.end_row();

                            ui.label("Port:");
                            ui.add_enabled(config_enabled, egui::DragValue::new(&mut self.port).clamp_range(1..=65535));
                            ui.end_row();

                            ui.label("Protocol:");
                            ui.add_enabled_ui(config_enabled, |ui| {
                                egui::ComboBox::from_label("")
                                    .selected_text(format!("{:?}", self.protocol))
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.protocol, Protocol::HTTP, "HTTP");
                                        ui.selectable_value(&mut self.protocol, Protocol::HTTPS, "HTTPS");
                                    })
                                    .response.on_hover_text("Select HTTP or HTTPS");
                            });
                            ui.end_row();
                        });

                    // Save/Load Buttons - Disable when server is not stopped/error
                    ui.horizontal(|ui| {
                        if ui.add_enabled(config_enabled, egui::Button::new("Save Config")).clicked() {
                            println!("Save Config button clicked"); // Placeholder
                        }

                        if ui.add_enabled(config_enabled, egui::Button::new("Load Config")).clicked() {
                            println!("Load Config button clicked"); // Placeholder
                        }
                    });
                }
                Tab::Monitoring => {
                    ui.heading("Server Monitoring");
                    
                    // Create a monitoring dashboard with multiple sections
                    ui.horizontal(|ui| {
                        // Left side - Server stats
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.heading("Server Statistics");
                                ui.separator();
                                
                                egui::Grid::new("stats_grid")
                                    .num_columns(2)
                                    .spacing([40.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label("Active Connections:");
                                        ui.label(format!("{}", self.server_stats.active_connections));
                                        ui.end_row();
                                        
                                        ui.label("Uptime:");
                                        ui.label(format!("{} seconds", self.server_stats.uptime_seconds));
                                        ui.end_row();
                                        
                                        // Add more stats as they become available
                                        ui.label("Memory Usage:");
                                        ui.label("N/A"); // Placeholder
                                        ui.end_row();
                                        
                                        ui.label("CPU Usage:");
                                        ui.label("N/A"); // Placeholder
                                        ui.end_row();
                                    });
                            });
                            
                            // Graph placeholder - will be replaced with actual data visualization
                            ui.group(|ui| {
                                ui.heading("Resource Usage");
                                ui.separator();
                                
                                let available_width = ui.available_width();
                                let graph_height = 100.0;
                                
                                // Placeholder for CPU usage graph
                                ui.label("CPU Usage Over Time");
                                ui.add(egui::widgets::ProgressBar::new(0.4)
                                    .animate(true)
                                    .desired_width(available_width));
                                
                                // Placeholder for Memory usage graph
                                ui.label("Memory Usage Over Time");
                                ui.add(egui::widgets::ProgressBar::new(0.6)
                                    .animate(true)
                                    .desired_width(available_width));
                                
                                // Add a refresh button
                                if ui.button("Refresh Statistics").clicked() {
                                    // Fetch latest stats here
                                    if let ServerStatus::Running = self.server_status {
                                        // Only update if server is running
                                        // self.server_stats = server::get_server_stats();
                                    }
                                }
                            });
                        });
                        
                        // Right side - Log viewer
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                ui.heading("Server Logs");
                                ui.separator();
                                
                                // Log filtering options
                                ui.horizontal(|ui| {
                                    ui.label("Filter:");
                                    let mut filter = String::new();
                                    ui.text_edit_singleline(&mut filter);
                                    
                                    ui.label("Level:");
                                    egui::ComboBox::from_label("")
                                        .selected_text("All")
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(&mut String::new(), "All".to_string(), "All");
                                            ui.selectable_value(&mut String::new(), "Info".to_string(), "Info");
                                            ui.selectable_value(&mut String::new(), "Warn".to_string(), "Warn");
                                            ui.selectable_value(&mut String::new(), "Error".to_string(), "Error");
                                        });
                                    
                                    if ui.button("Clear Logs").clicked() {
                                        self.logs.clear();
                                    }
                                });
                                
                                // Log display area with scrolling
                                egui::ScrollArea::vertical()
                                    .max_height(250.0)
                                    .show(ui, |ui| {
                                        for log in &self.logs {
                                            ui.label(log);
                                        }
                                        
                                        // Add some example logs if empty
                                        if self.logs.is_empty() {
                                            ui.weak("No logs to display");
                                        }
                                    });
                            });
                            
                            // Connection monitoring
                            ui.group(|ui| {
                                ui.heading("Connection Monitor");
                                ui.separator();
                                
                                ui.label(format!("Active Connections: {}", self.server_stats.active_connections));
                                
                                // Placeholder for connection list (will be replaced with actual data)
                                egui::ScrollArea::vertical()
                                    .max_height(150.0)
                                    .show(ui, |ui| {
                                        if self.server_status == ServerStatus::Running {
                                            ui.label("127.0.0.1 - [GET] /index.html - 200 OK");
                                            ui.label("192.168.1.5 - [POST] /api/data - 201 Created");
                                            ui.label("127.0.0.1 - [GET] /styles.css - 304 Not Modified");
                                        } else {
                                            ui.weak("Server not running");
                                        }
                                    });
                            });
                        });
                    });
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
    fn new(_cc: &eframe::CreationContext<'_>, tokio_handle: Handle) -> Self {
        Self {
            server_status: ServerStatus::Stopped,
            active_tab: Tab::Dashboard,
            host: "127.0.0.1".to_string(),
            port: 8000,
            protocol: Protocol::HTTP,
            tokio_handle,
            shutdown_sender: None,
            server_stats: ServerStats {
                active_connections: 0,
                uptime_seconds: 0,
            },
            logs: Vec::new(),
        }
    }
    
    // Add a method to add a log entry
    fn add_log(&mut self, message: &str) {
        // Add timestamp using std library instead of chrono for now
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let timestamp = format!("{:02}:{:02}:{:02}", 
            (now / 3600) % 24, 
            (now / 60) % 60, 
            now % 60);
        
        let log_entry = format!("[{}] {}", timestamp, message);
        
        self.logs.push(log_entry);
        
        // Limit the number of logs to prevent memory issues
        if self.logs.len() > 1000 {
            self.logs.remove(0);
        }
    }
    
    // Add a method to update stats periodically
    fn update_stats(&mut self) {
        // Only update if the server is running
        if let ServerStatus::Running = self.server_status {
            // This would eventually connect to the server to fetch actual stats
            // For now we'll just increment the uptime
            self.server_stats.uptime_seconds += 1;
            
            // In the future, this would call server::get_server_stats() 
            // instead of manually updating
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    // Create the Tokio runtime *before* the GUI
    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let handle = runtime.handle().clone(); // Get a handle to the runtime

    // Keep the runtime alive in the background (or manage its lifecycle appropriately)
    // For simplicity, we can leak it or run it in a separate thread if needed.
    // Here, we'll let the main thread own it until the GUI exits.

    // Start the GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    // Pass the handle to the app's constructor
    eframe::run_native(
        "Solara Server GUI",
        options,
        Box::new(move |cc| Box::new(SolaraApp::new(cc, handle.clone()))), // Clone handle for the closure
    )
    // The runtime will be dropped here when main exits
}

