mod server;

use eframe::egui;
use std::thread;

struct SolaraApp {
    server_status: String,
}

impl eframe::App for SolaraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Solara Server GUI");
            ui.label("Welcome!");
            ui.label(&self.server_status);
        });
    }
}

impl SolaraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            server_status: "Server running at http://localhost:8000".to_string(),
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