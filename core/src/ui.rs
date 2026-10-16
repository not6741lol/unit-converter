//! The egui UI, shared by the native `egui` front-end and the WebAssembly build.
//!
//! Both front-ends call [`draw`]; all behaviour lives in [`crate::app::App`].

use crate::app::{App, Entry};
use crate::{Mode, NumFormat};

/// Draw the whole app into `ui`.
pub fn draw(app: &mut App, ui: &mut egui::Ui) {
    let theme = if app.dark {
        egui::ThemePreference::Dark
    } else {
        egui::ThemePreference::Light
    };
    ui.ctx().set_theme(theme);

    egui::ScrollArea::vertical().show(ui, |ui| {
        top_bar(app, ui);
        ui.separator();
        match app.mode {
            Mode::Convert => convert_ui(app, ui),
            Mode::Calculator => calculator_ui(app, ui),
        }
        ui.add_space(4.0);
        ui.separator();
        saved_lists(app, ui);
        ui.add_space(6.0);
    });
}

/// Mode switch on the left, theme switch on the right.
fn top_bar(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let mut mode = app.mode;
        ui.selectable_value(&mut mode, Mode::Convert, "Convert");
        ui.selectable_value(&mut mode, Mode::Calculator, "Calculator");
        if mode != app.mode {
            app.set_mode(mode);
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if app.dark { "Light" } else { "Dark" };
            if ui.button(label).clicked() {
                app.toggle_dark();
            }
        });
    });
}

fn convert_ui(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Search");
        ui.add(
            egui::TextEdit::singleline(&mut app.search)
                .hint_text("filter categories")
                .desired_width(200.0),
        );
    });

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Type");
        let filtered = app.filtered_categories();
        let mut chosen = None;
        egui::ComboBox::from_id_salt("category")
            .selected_text(app.category.label())
            .show_ui(ui, |ui| {
                for c in filtered {
                    if ui.selectable_label(app.category == c, c.label()).clicked() {
                        chosen = Some(c);
                    }
                }
            });
        if let Some(c) = chosen {
            app.set_category(c);
        }
    });

    ui.add_space(6.0);
    let units = app.units();
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("from")
            .selected_text(app.from_unit())
            .width(140.0)
            .show_ui(ui, |ui| {
                for (i, u) in units.iter().enumerate() {
                    ui.selectable_value(&mut app.from, i, *u);
                }
            });

        if ui.button("Swap").clicked() {
            app.swap();
        }

        egui::ComboBox::from_id_salt("to")
            .selected_text(app.to_unit())
            .width(140.0)
            .show_ui(ui, |ui| {
                for (i, u) in units.iter().enumerate() {
                    ui.selectable_value(&mut app.to, i, *u);
                }
            });
    });

    value_row(app, ui);
    format_row(app, ui);
    result_row(app, ui);
}

fn calculator_ui(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.label("Expression");
    ui.add(
        egui::TextEdit::singleline(&mut app.input)
            .hint_text("e.g. 2+3*4, sqrt(2), sin(pi/2)")
            .desired_width(f32::INFINITY),
    );
    format_row(app, ui);
    result_row(app, ui);
}

fn value_row(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Value");
        ui.add(
            egui::TextEdit::singleline(&mut app.input)
                .hint_text("number or expression")
                .desired_width(220.0),
        );
    });
}

fn format_row(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label("Format");
        egui::ComboBox::from_id_salt("format")
            .selected_text(app.format.label())
            .show_ui(ui, |ui| {
                for f in NumFormat::ALL {
                    ui.selectable_value(&mut app.format, f, f.label());
                }
            });
    });
}

fn result_row(app: &mut App, ui: &mut egui::Ui) {
    ui.add_space(8.0);
    let text = app.result_text();
    let shown = if text.is_empty() {
        "=".to_string()
    } else {
        format!("= {text}")
    };

    ui.horizontal_wrapped(|ui| {
        ui.label(egui::RichText::new(shown).size(16.0));
    });

    if !text.is_empty() {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if ui.button("Copy").clicked() {
                ui.ctx().copy_text(text.clone());
            }
            if ui.button("Save").clicked() {
                app.save();
            }
            let fav_label = if app.is_favorite() { "Unfav" } else { "Fav" };
            if ui.button(fav_label).clicked() {
                app.toggle_favorite();
            }
        });
    }
}

fn saved_lists(app: &mut App, ui: &mut egui::Ui) {
    let history = app.history().to_vec();
    let favorites = app.favorites().to_vec();
    ui.add_space(8.0);
    entry_list(ui, "History", &history, true, app);
    ui.add_space(8.0);
    entry_list(ui, "Favorites", &favorites, false, app);
}

/// A titled, clickable list of saved conversions.
fn entry_list(ui: &mut egui::Ui, title: &str, entries: &[Entry], history: bool, app: &mut App) {
    let mut restore: Option<Entry> = None;
    let mut clear = false;

    ui.horizontal(|ui| {
        ui.strong(format!("{title} ({})", entries.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !entries.is_empty() && ui.small_button("Clear").clicked() {
                clear = true;
            }
        });
    });

    if entries.is_empty() {
        ui.weak("nothing saved yet");
    } else {
        for entry in entries {
            if ui.selectable_label(false, entry.describe()).clicked() {
                restore = Some(entry.clone());
            }
        }
    }

    if clear {
        if history {
            app.clear_history();
        } else {
            app.clear_favorites();
        }
    }
    if let Some(entry) = restore {
        app.restore(&entry);
    }
}
