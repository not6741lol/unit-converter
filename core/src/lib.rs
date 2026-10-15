//! Shared conversion logic used by every GUI front-end (`egui`, `fltk`, `win32`).
//!
//! Keeping this in one place means all three front-ends produce identical
//! results and are covered by the same test suite.

pub mod math;

/// The kinds of measurement the converter understands.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Category {
    Length,
    Weight,
    Temperature,
}

/// All categories, in display order.
pub const ALL_CATEGORIES: [Category; 3] = [
    Category::Length,
    Category::Weight,
    Category::Temperature,
];

impl Category {
    /// The units for this category, in display order.
    pub fn units(self) -> &'static [&'static str] {
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
            Category::Weight => &["Gram", "Kilogram", "Milligram", "Pound", "Ounce", "Tonne"],
            Category::Temperature => &["Celsius", "Fahrenheit", "Kelvin"],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Weight => "Weight",
            Category::Temperature => "Temperature",
        }
    }
}

/// Size of one unit, measured in the category's base unit
/// (metre for length, gram for weight).
pub fn factor(category: Category, unit: &str) -> f64 {
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

/// Temperature can't use the factor trick because it has offsets, not just
/// scale. So we funnel everything through Celsius.
pub fn to_celsius(unit: &str, v: f64) -> f64 {
    match unit {
        "Fahrenheit" => (v - 32.0) * 5.0 / 9.0,
        "Kelvin" => v - 273.15,
        _ => v, // Celsius
    }
}

pub fn from_celsius(unit: &str, c: f64) -> f64 {
    match unit {
        "Fahrenheit" => c * 9.0 / 5.0 + 32.0,
        "Kelvin" => c + 273.15,
        _ => c, // Celsius
    }
}

/// Convert `value` from unit `from` to unit `to` within `category`.
pub fn convert(category: Category, from: &str, to: &str, value: f64) -> f64 {
    if category == Category::Temperature {
        from_celsius(to, to_celsius(from, value))
    } else {
        value * factor(category, from) / factor(category, to)
    }
}

/// Trim trailing zeros so `1000.000000` reads as `1000`.
pub fn fmt_num(n: f64) -> String {
    if !n.is_finite() {
        return "-".to_string();
    }
    if n == 0.0 {
        return "0".to_string();
    }
    // Very small (but non-zero) values would round to "0" with six fixed
    // decimals, which is misleading. Show them in scientific notation instead.
    if n.abs() < 1e-6 {
        return format!("{n:e}");
    }
    let s = format!("{n:.6}");
    let trimmed = s.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-" || trimmed == "-0" {
        "0".to_string()
    } else {
        trimmed.to_string()
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
        assert!(close(conv(Category::Length, "Foot", "Inch", 1.0), 12.0));
        assert!(close(conv(Category::Length, "Yard", "Foot", 1.0), 3.0));
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
        // Small non-zero values must not collapse to "0".
        assert_eq!(fmt_num(1e-9), "1e-9");
        assert_eq!(fmt_num(-0.0000004), "-4e-7");
        assert_ne!(fmt_num(1e-9), "0");
    }
}
