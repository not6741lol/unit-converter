// In release builds, hide the console window on Windows (run as a GUI app).
// Debug builds keep the console so `println!` and panics are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

mod math;

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

#[derive(PartialEq, Eq, Clone, Copy)]
enum Category {
    Length,
    Weight,
    Temperature,
}

impl Category {
    // Every category except temperature shares the same "convert via a base
    // unit" trick, so all we need per unit is its size relative to the base.
    fn units(self) -> &'static [&'static str] {
        match self {
            Category::Length => &[
                "Meter",
                "Kilometer",
                "Centimeter",
                "Millimeter",
                "Mile",
                "Yard",
                "Foot",
                "Inch",
            ],
            Category::Weight => &[
                "Gram",
                "Kilogram",
                "Milligram",
                "Pound",
                "Ounce",
                "Tonne",
            ],
            Category::Temperature => &["Celsius", "Fahrenheit", "Kelvin"],
        }
    }

    fn label(self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Weight => "Weight",
            Category::Temperature => "Temperature",
        }
    }
}

const ALL_CATEGORIES: [Category; 3] = [
    Category::Length,
    Category::Weight,
    Category::Temperature,
];

// Size of one unit, measured in the category's base unit
// (metre for length, gram for weight).
fn factor(category: Category, unit: &str) -> f64 {
    match category {
        Category::Length => match unit {
            "Kilometer" => 1000.0,
            "Centimeter" => 0.01,
            "Millimeter" => 0.001,
            "Mile" => 1609.344,
            "Yard" => 0.9144,
            "Foot" => 0.3048,
            "Inch" => 0.0254,
            _ => 1.0, // Meter
        },
        Category::Weight => match unit {
            "Kilogram" => 1000.0,
            "Milligram" => 0.001,
            "Pound" => 453.59237,
            "Ounce" => 28.349_523_125,
            "Tonne" => 1_000_000.0,
            _ => 1.0, // Gram
        },
        Category::Temperature => 1.0,
    }
}

// Temperature can't use the factor trick because it has offsets, not just
// scale. So we funnel everything through Celsius.
fn to_celsius(unit: &str, v: f64) -> f64 {
    match unit {
        "Fahrenheit" => (v - 32.0) * 5.0 / 9.0,
        "Kelvin" => v - 273.15,
        _ => v, // Celsius
    }
}

fn from_celsius(unit: &str, c: f64) -> f64 {
    match unit {
        "Fahrenheit" => c * 9.0 / 5.0 + 32.0,
        "Kelvin" => c + 273.15,
        _ => c, // Celsius
    }
}

fn convert(category: Category, from: &str, to: &str, value: f64) -> f64 {
    if category == Category::Temperature {
        from_celsius(to, to_celsius(from, value))
    } else {
        value * factor(category, from) / factor(category, to)
    }
}

// Trim trailing zeros so 1000.000000 reads as 1000.
fn fmt_num(n: f64) -> String {
    if !n.is_finite() {
        return "-".to_string();
    }
    let s = format!("{n:.6}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" || trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
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
        self.from_index = self.from_index.min(units.len() - 1);
        self.to_index = self.to_index.min(units.len() - 1);

        {
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
                    self.to_index = 1.min(self.category.units().len() - 1);
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

                if ui.button("⇄").clicked() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn conv(category: Category, from: &str, to: &str, v: f64) -> f64 {
        convert(category, from, to, v)
    }

    #[test]
    fn length_conversions() {
        assert!(close(conv(Category::Length, "Kilometer", "Meter", 1.0), 1000.0));
        assert!(close(conv(Category::Length, "Meter", "Kilometer", 1000.0), 1.0));
        assert!(close(conv(Category::Length, "Mile", "Meter", 1.0), 1609.344));
        assert!(close(conv(Category::Length, "Meter", "Centimeter", 1.0), 100.0));
        // 1 foot = 12 inches, 1 yard = 3 feet
        assert!(close(conv(Category::Length, "Foot", "Inch", 1.0), 12.0));
        assert!(close(conv(Category::Length, "Yard", "Foot", 1.0), 3.0));
        // 1 inch = 2.54 cm
        assert!(close(conv(Category::Length, "Inch", "Centimeter", 1.0), 2.54));
    }

    #[test]
    fn weight_conversions() {
        assert!(close(conv(Category::Weight, "Kilogram", "Gram", 1.0), 1000.0));
        assert!(close(conv(Category::Weight, "Tonne", "Kilogram", 1.0), 1000.0));
        assert!(close(conv(Category::Weight, "Pound", "Gram", 1.0), 453.59237));
        assert!(close(conv(Category::Weight, "Pound", "Ounce", 1.0), 16.0));
        assert!(close(conv(Category::Weight, "Milligram", "Gram", 1000.0), 1.0));
    }

    #[test]
    fn temperature_conversions() {
        assert!(close(conv(Category::Temperature, "Celsius", "Fahrenheit", 100.0), 212.0));
        assert!(close(conv(Category::Temperature, "Celsius", "Fahrenheit", 0.0), 32.0));
        assert!(close(conv(Category::Temperature, "Fahrenheit", "Celsius", 212.0), 100.0));
        // The famous crossover point
        assert!(close(conv(Category::Temperature, "Celsius", "Fahrenheit", -40.0), -40.0));
        assert!(close(conv(Category::Temperature, "Celsius", "Kelvin", 0.0), 273.15));
        assert!(close(conv(Category::Temperature, "Kelvin", "Celsius", 273.15), 0.0));
        assert!(close(conv(Category::Temperature, "Fahrenheit", "Kelvin", 32.0), 273.15));
    }

    #[test]
    fn same_unit_is_identity() {
        for c in ALL_CATEGORIES {
            for u in c.units() {
                assert!(close(conv(c, u, u, 42.5), 42.5), "identity failed for {u}");
            }
        }
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt_num(1000.0), "1000");
        assert_eq!(fmt_num(2.5), "2.5");
        assert_eq!(fmt_num(0.0), "0");
        assert_eq!(fmt_num(-0.0), "0");
        assert_eq!(fmt_num(1234.5678), "1234.5678");
    }
}
