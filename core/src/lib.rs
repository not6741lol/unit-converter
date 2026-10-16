//! Shared conversion logic used by every GUI front-end
//! (`egui`, `fltk`, `win32`, `web`).
//!
//! Keeping this in one place means all front-ends produce identical results and
//! are covered by the same test suite.

pub mod app;
pub mod math;
#[cfg(feature = "ui")]
pub mod ui;

/// The kinds of measurement the converter understands.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Category {
    Length,
    Area,
    Volume,
    Weight,
    Speed,
    Acceleration,
    Time,
    Frequency,
    Temperature,
    Pressure,
    Energy,
    Power,
    Force,
    Angle,
    Illuminance,
    Data,
}

/// All categories, in display order.
pub const ALL_CATEGORIES: [Category; 16] = [
    Category::Length,
    Category::Area,
    Category::Volume,
    Category::Weight,
    Category::Speed,
    Category::Acceleration,
    Category::Time,
    Category::Frequency,
    Category::Temperature,
    Category::Pressure,
    Category::Energy,
    Category::Power,
    Category::Force,
    Category::Angle,
    Category::Illuminance,
    Category::Data,
];

/// Which of the two things the app is doing.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    /// Convert a value between units.
    Convert,
    /// Plain arithmetic, no units at all.
    Calculator,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Convert, Mode::Calculator];

    pub fn label(self) -> &'static str {
        match self {
            Mode::Convert => "Convert",
            Mode::Calculator => "Calculator",
        }
    }
}

/// How many decimals to show (or scientific notation).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NumFormat {
    Auto,
    Dec0,
    Dec1,
    Dec2,
    Dec3,
    Dec4,
    Dec6,
    Scientific,
}

impl NumFormat {
    pub const ALL: [NumFormat; 8] = [
        NumFormat::Auto,
        NumFormat::Dec0,
        NumFormat::Dec1,
        NumFormat::Dec2,
        NumFormat::Dec3,
        NumFormat::Dec4,
        NumFormat::Dec6,
        NumFormat::Scientific,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NumFormat::Auto => "Auto",
            NumFormat::Dec0 => "0",
            NumFormat::Dec1 => "1",
            NumFormat::Dec2 => "2",
            NumFormat::Dec3 => "3",
            NumFormat::Dec4 => "4",
            NumFormat::Dec6 => "6",
            NumFormat::Scientific => "Sci",
        }
    }
}

impl Category {
    /// The units for this category, in display order.
    pub fn units(self) -> &'static [&'static str] {
        match self {
            Category::Length => &[
                "Meter",
                "Kilometer",
                "Centimeter",
                "Millimeter",
                "Micrometer",
                "Nanometer",
                "Mile",
                "Yard",
                "Foot",
                "Inch",
                "Nautical Mile",
            ],
            Category::Area => &[
                "Square Meter",
                "Square Kilometer",
                "Square Centimeter",
                "Square Millimeter",
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
                "Cubic Centimeter",
                "Gallon (US)",
                "Gallon (UK)",
                "Quart (US)",
                "Pint (US)",
                "Cup (US)",
                "Fluid Ounce (US)",
                "Tablespoon",
                "Teaspoon",
            ],
            Category::Weight => &[
                "Gram",
                "Kilogram",
                "Milligram",
                "Microgram",
                "Pound",
                "Ounce",
                "Stone",
                "Tonne",
                "US Ton",
            ],
            Category::Speed => &[
                "Meter/second",
                "Kilometer/hour",
                "Mile/hour",
                "Knot",
                "Foot/second",
                "Mach",
            ],
            Category::Acceleration => &[
                "Meter/second²",
                "Foot/second²",
                "Standard gravity",
                "Galileo",
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
                "Month",
                "Year",
            ],
            Category::Frequency => &["Hertz", "Kilohertz", "Megahertz", "Gigahertz", "RPM"],
            Category::Temperature => &["Celsius", "Fahrenheit", "Kelvin", "Rankine"],
            Category::Pressure => &[
                "Pascal",
                "Hectopascal",
                "Kilopascal",
                "Megapascal",
                "Bar",
                "Millibar",
                "PSI",
                "Atmosphere",
                "Torr",
            ],
            Category::Energy => &[
                "Joule",
                "Kilojoule",
                "Megajoule",
                "Calorie",
                "Kilocalorie",
                "Watt-hour",
                "Kilowatt-hour",
                "Electronvolt",
                "BTU",
                "Foot-pound",
            ],
            Category::Power => &[
                "Watt",
                "Kilowatt",
                "Megawatt",
                "Gigawatt",
                "Horsepower (mech)",
                "Horsepower (metric)",
                "BTU/hour",
            ],
            Category::Force => &[
                "Newton",
                "Kilonewton",
                "Pound-force",
                "Ounce-force",
                "Dyne",
                "Kilogram-force",
            ],
            Category::Angle => &[
                "Degree",
                "Radian",
                "Gradian",
                "Turn",
                "Arcminute",
                "Arcsecond",
            ],
            Category::Illuminance => &["Lux", "Foot-candle", "Phot", "Nox"],
            Category::Data => &[
                "Byte", "Bit", "Kilobit", "Kilobyte", "Kibibyte", "Megabyte", "Mebibyte",
                "Gigabyte", "Gibibyte", "Terabyte", "Tebibyte", "Petabyte",
            ],
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Category::Length => "Length",
            Category::Area => "Area",
            Category::Volume => "Volume",
            Category::Weight => "Weight",
            Category::Speed => "Speed",
            Category::Acceleration => "Acceleration",
            Category::Time => "Time",
            Category::Frequency => "Frequency",
            Category::Temperature => "Temperature",
            Category::Pressure => "Pressure",
            Category::Energy => "Energy",
            Category::Power => "Power",
            Category::Force => "Force",
            Category::Angle => "Angle",
            Category::Illuminance => "Illuminance",
            Category::Data => "Data",
        }
    }
}

/// Size of one unit, measured in the category's base unit.
///
/// Bases: metre (Length), square metre (Area), litre (Volume), gram (Weight),
/// metre per second (Speed), metre per second squared (Acceleration), second
/// (Time), hertz (Frequency), pascal (Pressure), joule (Energy), watt (Power),
/// newton (Force), radian (Angle), lux (Illuminance), byte (Data).
/// Temperature is the exception and is handled separately in [`convert`].
pub fn factor(category: Category, unit: &str) -> f64 {
    use std::f64::consts::{PI, TAU};

    match category {
        Category::Length => match unit {
            "Kilometer" => 1000.0,
            "Centimeter" => 0.01,
            "Millimeter" => 0.001,
            "Micrometer" => 1e-6,
            "Nanometer" => 1e-9,
            "Mile" => 1609.344,
            "Yard" => 0.9144,
            "Foot" => 0.3048,
            "Inch" => 0.0254,
            "Nautical Mile" => 1852.0,
            _ => 1.0, // Meter
        },
        Category::Area => match unit {
            "Square Kilometer" => 1_000_000.0,
            "Square Centimeter" => 0.0001,
            "Square Millimeter" => 1e-6,
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
            "Cubic Centimeter" => 0.001,
            "Gallon (US)" => 3.785411784,
            "Gallon (UK)" => 4.54609,
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
            "Microgram" => 1e-6,
            "Pound" => 453.59237,
            "Ounce" => 28.349523125,
            "Stone" => 6350.29318,
            "Tonne" => 1_000_000.0,
            "US Ton" => 907_184.74,
            _ => 1.0, // Gram
        },
        Category::Speed => match unit {
            "Kilometer/hour" => 1.0 / 3.6,
            "Mile/hour" => 0.44704,
            "Knot" => 0.5144444444444444,
            "Foot/second" => 0.3048,
            "Mach" => 340.29,
            _ => 1.0, // Meter/second
        },
        Category::Acceleration => match unit {
            "Foot/second²" => 0.3048,
            "Standard gravity" => 9.80665,
            "Galileo" => 0.01,
            _ => 1.0, // Meter/second²
        },
        Category::Time => match unit {
            "Nanosecond" => 1e-9,
            "Microsecond" => 1e-6,
            "Millisecond" => 1e-3,
            "Minute" => 60.0,
            "Hour" => 3600.0,
            "Day" => 86_400.0,
            "Week" => 604_800.0,
            "Month" => 2_629_746.0, // average Gregorian month
            "Year" => 31_556_952.0, // average Gregorian year
            _ => 1.0,               // Second
        },
        Category::Frequency => match unit {
            "Kilohertz" => 1e3,
            "Megahertz" => 1e6,
            "Gigahertz" => 1e9,
            "RPM" => 1.0 / 60.0,
            _ => 1.0, // Hertz
        },
        Category::Pressure => match unit {
            "Hectopascal" => 100.0,
            "Kilopascal" => 1000.0,
            "Megapascal" => 1e6,
            "Bar" => 100_000.0,
            "Millibar" => 100.0,
            "PSI" => 6894.757293168361,
            "Atmosphere" => 101_325.0,
            "Torr" => 133.32236842105263,
            _ => 1.0, // Pascal
        },
        Category::Energy => match unit {
            "Kilojoule" => 1000.0,
            "Megajoule" => 1e6,
            "Calorie" => 4.184,
            "Kilocalorie" => 4184.0,
            "Watt-hour" => 3600.0,
            "Kilowatt-hour" => 3.6e6,
            "Electronvolt" => 1.602176634e-19,
            "BTU" => 1055.05585262,
            "Foot-pound" => 1.3558179483314004,
            _ => 1.0, // Joule
        },
        Category::Power => match unit {
            "Kilowatt" => 1e3,
            "Megawatt" => 1e6,
            "Gigawatt" => 1e9,
            "Horsepower (mech)" => 745.6998715822702,
            "Horsepower (metric)" => 735.49875,
            "BTU/hour" => 0.2930710701722222,
            _ => 1.0, // Watt
        },
        Category::Force => match unit {
            "Kilonewton" => 1000.0,
            "Pound-force" => 4.4482216152605,
            "Ounce-force" => 4.4482216152605 / 16.0,
            "Dyne" => 1e-5,
            "Kilogram-force" => 9.80665,
            _ => 1.0, // Newton
        },
        // Base here is the radian; `Degree` is first in the list because it is
        // what people reach for most often.
        Category::Angle => match unit {
            "Radian" => 1.0,
            "Gradian" => PI / 200.0,
            "Turn" => TAU,
            "Arcminute" => PI / 10_800.0,
            "Arcsecond" => PI / 648_000.0,
            _ => PI / 180.0, // Degree
        },
        Category::Illuminance => match unit {
            "Foot-candle" => 10.763910416709722,
            "Phot" => 10_000.0,
            "Nox" => 0.001,
            _ => 1.0, // Lux
        },
        Category::Data => match unit {
            "Bit" => 0.125,
            "Kilobit" => 125.0,
            "Kilobyte" => 1e3,
            "Kibibyte" => 1024.0,
            "Megabyte" => 1e6,
            "Mebibyte" => 1_048_576.0,
            "Gigabyte" => 1e9,
            "Gibibyte" => 1_073_741_824.0,
            "Terabyte" => 1e12,
            "Tebibyte" => 1_099_511_627_776.0,
            "Petabyte" => 1e15,
            _ => 1.0, // Byte
        },
        // Temperature is handled by `to_celsius` / `from_celsius`.
        Category::Temperature => 1.0,
    }
}

/// Temperature can't use the factor trick because it has offsets, not just
/// scale. So we funnel everything through Celsius.
pub fn to_celsius(unit: &str, v: f64) -> f64 {
    match unit {
        "Fahrenheit" => (v - 32.0) * 5.0 / 9.0,
        "Kelvin" => v - 273.15,
        "Rankine" => (v - 491.67) * 5.0 / 9.0,
        _ => v, // Celsius
    }
}

pub fn from_celsius(unit: &str, c: f64) -> f64 {
    match unit {
        "Fahrenheit" => c * 9.0 / 5.0 + 32.0,
        "Kelvin" => c + 273.15,
        "Rankine" => c * 9.0 / 5.0 + 491.67,
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

/// Case-insensitive "does `haystack` contain `needle`?" (empty needle matches).
pub fn matches(haystack: &str, needle: &str) -> bool {
    if needle.trim().is_empty() {
        return true;
    }
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}

/// Format `n` using the chosen [`NumFormat`].
pub fn fmt_with(n: f64, fmt: NumFormat) -> String {
    if !n.is_finite() {
        return "-".to_string();
    }
    match fmt {
        NumFormat::Auto => fmt_num(n),
        NumFormat::Scientific => format!("{n:e}"),
        NumFormat::Dec0 => format!("{n:.0}"),
        NumFormat::Dec1 => format!("{n:.1}"),
        NumFormat::Dec2 => format!("{n:.2}"),
        NumFormat::Dec3 => format!("{n:.3}"),
        NumFormat::Dec4 => format!("{n:.4}"),
        NumFormat::Dec6 => format!("{n:.6}"),
    }
}

/// Format a result for display: trim trailing zeros, and fall back to
/// scientific notation for very large/small magnitudes so they stay readable.
pub fn fmt_num(n: f64) -> String {
    if !n.is_finite() {
        return "-".to_string();
    }
    if n == 0.0 {
        return "0".to_string();
    }
    let a = n.abs();
    if a < 1e-6 || a >= 1e15 {
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
        assert!(close(
            conv(Category::Length, "Nautical Mile", "Meter", 1.0),
            1852.0
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
        assert!(close(
            conv(Category::Volume, "Gallon (UK)", "Liter", 1.0),
            4.54609
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
        assert!(close(conv(Category::Weight, "Stone", "Pound", 1.0), 14.0));
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
    fn acceleration_conversions() {
        assert!(close(
            conv(
                Category::Acceleration,
                "Standard gravity",
                "Meter/second²",
                1.0
            ),
            9.80665
        ));
        assert!(close(
            conv(Category::Acceleration, "Foot/second²", "Meter/second²", 1.0),
            0.3048
        ));
        assert!(close(
            conv(Category::Acceleration, "Galileo", "Meter/second²", 100.0),
            1.0
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
        assert!(close(conv(Category::Time, "Year", "Day", 1.0), 365.2425));
    }

    #[test]
    fn frequency_conversions() {
        assert!(close(
            conv(Category::Frequency, "Kilohertz", "Hertz", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Frequency, "Gigahertz", "Megahertz", 1.0),
            1000.0
        ));
        assert!(close(conv(Category::Frequency, "RPM", "Hertz", 60.0), 1.0));
    }

    #[test]
    fn pressure_conversions() {
        assert!(close(
            conv(Category::Pressure, "Bar", "Pascal", 1.0),
            100_000.0
        ));
        assert!(close(
            conv(Category::Pressure, "Atmosphere", "Pascal", 1.0),
            101_325.0
        ));
        assert!(close(
            conv(Category::Pressure, "Kilopascal", "Pascal", 1.0),
            1000.0
        ));
    }

    #[test]
    fn energy_conversions() {
        assert!(close(
            conv(Category::Energy, "Kilojoule", "Joule", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Energy, "Kilocalorie", "Calorie", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Energy, "Kilowatt-hour", "Joule", 1.0),
            3.6e6
        ));
        assert!(close(
            conv(Category::Energy, "Watt-hour", "Joule", 1.0),
            3600.0
        ));
    }

    #[test]
    fn power_conversions() {
        assert!(close(
            conv(Category::Power, "Kilowatt", "Watt", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Power, "Horsepower (metric)", "Watt", 1.0),
            735.49875
        ));
        assert!(close(
            conv(Category::Power, "Megawatt", "Kilowatt", 1.0),
            1000.0
        ));
    }

    #[test]
    fn force_conversions() {
        assert!(close(
            conv(Category::Force, "Kilonewton", "Newton", 1.0),
            1000.0
        ));
        assert!(close(
            conv(Category::Force, "Pound-force", "Newton", 1.0),
            4.4482216152605
        ));
        assert!(close(
            conv(Category::Force, "Kilogram-force", "Newton", 1.0),
            9.80665
        ));
    }

    #[test]
    fn angle_conversions() {
        assert!(close(conv(Category::Angle, "Turn", "Degree", 1.0), 360.0));
        assert!(close(
            conv(Category::Angle, "Degree", "Radian", 180.0),
            std::f64::consts::PI
        ));
        assert!(close(
            conv(Category::Angle, "Arcminute", "Degree", 60.0),
            1.0
        ));
        assert!(close(
            conv(Category::Angle, "Gradian", "Degree", 400.0),
            360.0
        ));
    }

    #[test]
    fn illuminance_conversions() {
        assert!(close(
            conv(Category::Illuminance, "Phot", "Lux", 1.0),
            10_000.0
        ));
        assert!(close(
            conv(Category::Illuminance, "Foot-candle", "Lux", 1.0),
            10.763910416709722
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
        assert!(close(
            conv(Category::Data, "Petabyte", "Terabyte", 1.0),
            1000.0
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
        assert!(close(
            conv(Category::Temperature, "Kelvin", "Rankine", 0.0),
            0.0
        ));
        assert!(close(
            conv(Category::Temperature, "Celsius", "Rankine", 0.0),
            491.67
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
        assert_eq!(fmt_num(1e20), "1e20");
        assert_ne!(fmt_num(1e-9), "0");
    }

    #[test]
    fn explicit_formats() {
        assert_eq!(fmt_with(2.5, NumFormat::Dec0), "2");
        assert_eq!(fmt_with(2.5, NumFormat::Dec2), "2.50");
        assert_eq!(fmt_with(1000.0, NumFormat::Dec2), "1000.00");
        assert_eq!(fmt_with(1234.5678, NumFormat::Dec1), "1234.6");
        assert_eq!(fmt_with(0.5, NumFormat::Scientific), "5e-1");
    }

    #[test]
    fn search_matching() {
        assert!(matches("Kilometer", "kilo"));
        assert!(matches("Kilometer", "METER"));
        assert!(matches("Kilometer", ""));
        assert!(!matches("Kilometer", "zzz"));
    }
}
