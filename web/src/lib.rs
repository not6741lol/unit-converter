//! Web (WASM) build of Convert — the same egui UI as the desktop front-end,
//! compiled to WebAssembly and started from `index.html`.
//!
//! Everything is gated to `wasm32` so the desktop workspace build ignores it.

#![cfg(target_arch = "wasm32")]

use convert_core::{ALL_CATEGORIES, Category, convert, fmt_num, math};
use eframe::egui;
use wasm_bindgen::prelude::*;

struct Converter {
    category: Category,
    from_index: usize,
    to_index: usize,
    input: String,
}

impl Default for Converter {
    fn default() -> Self {
        Self {
            category: Category::Length,
            from_index: 0,
            to_index: 1,
            input: "1".to_string(),
        }
    }
}

impl eframe::App for Converter {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let units = self.category.units();
        let last = units.len().saturating_sub(1);
        self.from_index = self.from_index.min(last);
        self.to_index = self.to_index.min(last);

        ui.add_space(6.0);
        ui.heading("Convert");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Type");
            let mut switched = false;
            egui::ComboBox::from_id_salt("category")
                .selected_text(self.category.label())
                .show_ui(ui, |ui| {
                    for c in ALL_CATEGORIES {
                        if ui.selectable_label(self.category == c, c.label()).clicked()
                            && self.category != c
                        {
                            self.category = c;
                            switched = true;
                        }
                    }
                });
            if switched {
                self.from_index = 0;
                self.to_index = 1.min(self.category.units().len().saturating_sub(1));
            }
        });

        ui.add_space(8.0);

        let units = self.category.units();
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("from")
                .selected_text(units[self.from_index])
                .show_ui(ui, |ui| {
                    for (i, u) in units.iter().enumerate() {
                        ui.selectable_value(&mut self.from_index, i, *u);
                    }
                });

            if ui.button("Swap").clicked() {
                std::mem::swap(&mut self.from_index, &mut self.to_index);
            }

            egui::ComboBox::from_id_salt("to")
                .selected_text(units[self.to_index])
                .show_ui(ui, |ui| {
                    for (i, u) in units.iter().enumerate() {
                        ui.selectable_value(&mut self.to_index, i, *u);
                    }
                });
        });

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Value");
            ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("number or expression")
                    .desired_width(180.0),
            );
        });

        ui.add_space(12.0);

        match math::eval(&self.input) {
            Ok(v) => {
                let result = convert(
                    self.category,
                    units[self.from_index],
                    units[self.to_index],
                    v,
                );
                let text = format!("{} {}", fmt_num(result), units[self.to_index]);
                ui.horizontal(|ui| {
                    ui.label(format!("= {text}"));
                    if ui.button("Copy").clicked() {
                        ui.ctx().copy_text(text.clone());
                    }
                });
            }
            Err(_) => {
                if self.input.trim().is_empty() {
                    ui.label("= ");
                } else {
                    ui.label("= (invalid expression)");
                }
            }
        }
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
