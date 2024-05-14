use nes::frontend::egui::EGuiApp;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "NES",
        native_options,
        Box::new(|_cc| Box::new(EGuiApp::new())),
    );
}
