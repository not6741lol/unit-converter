//! Web (WASM) build of Convert — the same UI as the native egui front-end,
//! compiled to WebAssembly and started from `index.html`.
//!
//! Everything is gated to `wasm32` so the desktop workspace build ignores it.

#![cfg(target_arch = "wasm32")]

use convert_core::app::App;
use wasm_bindgen::prelude::*;

/// Thin shell: the UI itself lives in `convert_core::ui`, shared with the
/// native egui build.
#[derive(Default)]
struct Converter {
    core: App,
}

impl eframe::App for Converter {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        convert_core::ui::draw(&mut self.core, ui);
    }
}

/// Handle to the web app, exposed to JavaScript via `wasm-bindgen`.
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

impl Default for WebHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WebHandle {
    /// Create a new handle and install the console logger.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        eframe::WebLogger::init(log::LevelFilter::Info).ok();
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    /// Start the app on the given canvas. Call once from JavaScript.
    #[wasm_bindgen]
    pub async fn start(&self, canvas: web_sys::HtmlCanvasElement) -> Result<(), JsValue> {
        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_cc| Ok(Box::new(Converter::default()))),
            )
            .await
    }

    /// Shut the app down and clean up.
    #[wasm_bindgen]
    pub fn destroy(&self) {
        self.runner.destroy();
    }
}
