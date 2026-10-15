//! Shared conversion logic used by every GUI front-end (`egui`, `fltk`, `win32`).
//!
//! Keeping this in one place means all three front-ends produce identical
//! results and are covered by the same test suite.

pub mod math;

/// The kinds of measurement the converter understands.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Category {
    Length,
    Area,
    Volume,
    Weight,
    Speed,
    Time,
    Data,
    Temperature,
}

/// All categories, in display order.
pub const ALL_CATEGORIES: [Category; 8] = [
    Category::Length,
    Category::Area,
    Category::Volume,
    Category::Weight,
    Category::Speed,
    Category::Time,
    Category::Data,
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
            Category::Area => &[
                "Square Meter",
                "Square Kilometer",
                "Square Centimeter",
                "Hectare",
                "Acre",
                "Square Mile",
                "Square Yard",
                "Square Foot",
                "Square Inch",
            ],
            Category::Volume => &[
                "Liter",
                "Milliliter",
                "Cubic Meter",
                "Gallon (US)",
                "Quart (US)",
                "Pint (US)",
                "Cup (US)",
                "Fluid Ounce (US)",
                "Tablespoon",
                "Teaspoon",
            ],
            Category::Weight => &["Gram", "Kilogram", "Milligram", "Pound", "Ounce", "Tonne"],
            Category::Speed => &[
                "Meter/second",
                "Kilometer/hour",
                "Mile/hour",
                "Knot",
                "Foot/second",
            ],
            Category::Time => &[
                "Second",
                "Nanosecond",
                "Microsecond",
                "Millisecond",
                "Minute",
                "Hour",
                "Day",
                "Week",
            ],
            Category::Data => &[
                "Byte", "Bit", "Kilobyte", "Kibibyte", "Megabyte", "Mebibyte", "Gigabyte",
                "Gibibyte", "Terabyte", "Tebibyte",
            ],
            Category::Temperature => &["Celsius", "Fahrenheit", "Kelvin"],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Area => "Area",
            Category::Volume => "Volume",
            Category::Weight => "Weight",
            Category::Speed => "Speed",
            Category::Time => "Time",
            Category::Data => "Data",
            Category::Temperature => "Temperature",
        }
    }
}

/// Size of one unit, measured in the category's base unit.
///
/// Bases: metre (Length), square metre (Area), litre (Volume), gram (Weight),
/// metre per second (Speed), second (Time), byte (Data). Temperature is the
/// exception and is handled separately in [`convert`].
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
        Category::Area => match unit {
            "Square Kilometer" => 1_000_000.0,
            "Square Centimeter" => 0.0001,
            "Hectare" => 10_000.0,
            "Acre" => 4046.8564224,
            "Square Mile" => 2_589_988.110336,
            "Square Yard" => 0.83612736,
            "Square Foot" => 0.09290304,
            "Square Inch" => 0.00064516,
            _ => 1.0, // Square Meter
        },
        Category::Volume => match unit {
            "Milliliter" => 0.001,
            "Cubic Meter" => 1000.0,
            "Gallon (US)" => 3.785411784,
            "Quart (US)" => 0.946352946,
            "Pint (US)" => 0.473176473,
            "Cup (US)" => 0.2365882365,
            "Fluid Ounce (US)" => 0.0295735295625,
            "Tablespoon" => 0.01478676478125,
            "Teaspoon" => 0.00492892159375,
            _ => 1.0, // Liter
        },
        Category::Weight => match unit {
            "Kilogram" => 1000.0,
            "Milligram" => 0.001,
            "Pound" => 453.59237,
            "Ounce" => 28.349523125,
            "Tonne" => 1_000_000.0,
            _ => 1.0, // Gram
        },
        Category::Speed => match unit {
            "Kilometer/hour" => 1.0 / 3.6,
            "Mile/hour" => 0.44704,
            "Knot" => 0.5144444444444444,
            "Foot/second" => 0.3048,
            _ => 1.0, // Meter/second
        },
        Category::Time => match unit {
            "Nanosecond" => 1e-9,
            "Microsecond" => 1e-6,
            "Millisecond" => 1e-3,
            "Minute" => 60.0,
            "Hour" => 3600.0,
            "Day" => 86_400.0,
            "Week" => 604_800.0,
            _ => 1.0, // Second
        },
        Category::Data => match unit {
            "Bit" => 0.125,
            "Kilobyte" => 1e3,
            "Kibibyte" => 1024.0,
            "Megabyte" => 1e6,
            "Mebibyte" => 1_048_576.0,
            "Gigabyte" => 1e9,
            "Gibibyte" => 1_073_741_824.0,
            "Terabyte" => 1e12,
            "Tebibyte" => 1_099_511_627_776.0,
            _ => 1.0, // Byte
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

/// Format a result for display: trim trailing zeros, and fall back to
/// scientific notation for tiny non-zero values so they don't read as `0`.
pub fn fmt_num(n: f64) -> String {
    if !n.is_finite() {
        return "-".to_string();
    }
    if n == 0.0 {
        return "0".to_string();
    }
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
        assert!(close(
            conv(Category::Length, "Kilometer", "Meter", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Length, "Meter", "Kilometer", 1000.0),
            1.0
        ));
        assert!(close(
            conv(Category::Length, "Mile", "Meter", 1.0),
            1609.344
        ));
        assert!(close(
            conv(Category::Length, "Meter", "Centimeter", 1.0),
            100.0
        ));
        assert!(close(conv(Category::Length, "Foot", "Inch", 1.0), 12.0));
        assert!(close(conv(Category::Length, "Yard", "Foot", 1.0), 3.0));
        assert!(close(
            conv(Category::Length, "Inch", "Centimeter", 1.0),
            2.54
        ));
    }

    #[test]
    fn area_conversions() {
        assert!(close(
            conv(Category::Area, "Hectare", "Square Meter", 1.0),
            10000.0
        ));
        assert!(close(
            conv(Category::Area, "Acre", "Square Meter", 1.0),
            4046.8564224
        ));
        assert!(close(
            conv(Category::Area, "Square Kilometer", "Hectare", 1.0),
            100.0
        ));
        assert!(close(
            conv(Category::Area, "Square Foot", "Square Inch", 1.0),
            144.0
        ));
    }

    #[test]
    fn volume_conversions() {
        assert!(close(
            conv(Category::Volume, "Liter", "Milliliter", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Volume, "Cubic Meter", "Liter", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Volume, "Gallon (US)", "Liter", 1.0),
            3.785411784
        ));
        assert!(close(
            conv(Category::Volume, "Cup (US)", "Fluid Ounce (US)", 1.0),
            8.0
        ));
        assert!(close(
            conv(Category::Volume, "Tablespoon", "Teaspoon", 1.0),
            3.0
        ));
    }

    #[test]
    fn weight_conversions() {
        assert!(close(
            conv(Category::Weight, "Kilogram", "Gram", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Weight, "Tonne", "Kilogram", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Weight, "Pound", "Gram", 1.0),
            453.59237
        ));
        assert!(close(conv(Category::Weight, "Pound", "Ounce", 1.0), 16.0));
        assert!(close(
            conv(Category::Weight, "Milligram", "Gram", 1000.0),
            1.0
        ));
    }

    #[test]
    fn speed_conversions() {
        assert!(close(
            conv(Category::Speed, "Kilometer/hour", "Meter/second", 3.6),
            1.0
        ));
        assert!(close(
            conv(Category::Speed, "Mile/hour", "Meter/second", 1.0),
            0.44704
        ));
        assert!(close(
            conv(Category::Speed, "Knot", "Meter/second", 1.0),
            0.5144444444444444
        ));
    }

    #[test]
    fn time_conversions() {
        assert!(close(conv(Category::Time, "Hour", "Minute", 1.0), 60.0));
        assert!(close(conv(Category::Time, "Day", "Second", 1.0), 86400.0));
        assert!(close(conv(Category::Time, "Week", "Day", 1.0), 7.0));
        assert!(close(
            conv(Category::Time, "Second", "Millisecond", 1.0),
            1000.0
        ));
    }

    #[test]
    fn data_conversions() {
        assert!(close(conv(Category::Data, "Byte", "Bit", 1.0), 8.0));
        assert!(close(conv(Category::Data, "Kibibyte", "Byte", 1.0), 1024.0));
        assert!(close(conv(Category::Data, "Kilobyte", "Byte", 1.0), 1000.0));
        assert!(close(
            conv(Category::Data, "Gibibyte", "Mebibyte", 1.0),
            1024.0
        ));
        assert!(close(
            conv(Category::Data, "Tebibyte", "Gibibyte", 1.0),
            1024.0
        ));
    }

    #[test]
    fn temperature_conversions() {
        assert!(close(
            conv(Category::Temperature, "Celsius", "Fahrenheit", 100.0),
            212.0
        ));
        assert!(close(
            conv(Category::Temperature, "Celsius", "Fahrenheit", 0.0),
            32.0
        ));
        assert!(close(
            conv(Category::Temperature, "Fahrenheit", "Celsius", 212.0),
            100.0
        ));
        assert!(close(
            conv(Category::Temperature, "Celsius", "Fahrenheit", -40.0),
            -40.0
        ));
        assert!(close(
            conv(Category::Temperature, "Celsius", "Kelvin", 0.0),
            273.15
        ));
        assert!(close(
            conv(Category::Temperature, "Kelvin", "Celsius", 273.15),
            0.0
        ));
        assert!(close(
            conv(Category::Temperature, "Fahrenheit", "Kelvin", 32.0),
            273.15
        ));
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
        assert_eq!(fmt_num(1e-9), "1e-9");
        assert_eq!(fmt_num(-0.0000004), "-4e-7");
        assert_ne!(fmt_num(1e-9), "0");
    }
}
