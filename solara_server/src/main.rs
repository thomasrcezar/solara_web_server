use eframe::egui;

struct SolaraApp {}

impl eframe::App for SolaraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Solara Server GUI");
            ui.label("Welcome!");
        });
    }
}

impl SolaraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {}
    }
}

fn main() -> Result<(), eframe::Error> {
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