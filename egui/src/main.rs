// In release builds, hide the console window on Windows (run as a GUI app).
// Debug builds keep the console so `println!` and panics are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use convert_core::{math, convert, fmt_num, Category, ALL_CATEGORIES};
use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([380.0, 260.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Convert",
        options,
        Box::new(|_cc| Ok(Box::new(Converter::default()))),
    )
}

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
        // Keep indices in range if the unit list changed length.
        let last = units.len().saturating_sub(1);
        self.from_index = self.from_index.min(last);
        self.to_index = self.to_index.min(last);

        ui.add_space(4.0);
        ui.heading("Convert");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Type");
            let mut switched = false;
            egui::ComboBox::from_id_salt("category")
                .selected_text(self.category.label())
                .show_ui(ui, |ui| {
                    for c in ALL_CATEGORIES {
                        if ui
                            .selectable_label(self.category == c, c.label())
                            .clicked()
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
                ui.label(format!("= {} {}", fmt_num(result), units[self.to_index]));
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
