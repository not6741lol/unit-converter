// In release builds, hide the console window on Windows (run as a GUI app).
// Debug builds keep the console so `println!` and panics are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use convert_core::app::App;
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([470.0, 560.0])
            .with_min_inner_size([430.0, 420.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Convert",
        options,
        Box::new(|_cc| Ok(Box::new(Converter::default()))),
    )
}

/// Thin shell: the UI itself lives in `convert_core::ui`, shared with the web build.
#[derive(Default)]
struct Converter {
    core: App,
}

impl eframe::App for Converter {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        convert_core::ui::draw(&mut self.core, ui);
    }
}
