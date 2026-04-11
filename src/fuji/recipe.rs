use std::fmt;

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

/// RAW -> JPEG conversion recipe.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub film: FilmSimulation,
    pub grain: GrainEffect,
    pub color_chrome: ColorChromeEffect,
    pub color_chrome_blue: ColorChromeEffect,
    pub white_balance: WhiteBalance,
    pub white_balance_red: i32,
    pub white_balance_blue: i32,
    pub dynamic_range: DynamicRange,
    pub dynamic_range_priority: DynamicRangePriority,
    #[serde(with = "third_stops")]
    pub exposure: f64,
    #[serde(with = "tenth_stops")]
    pub highlight: f64,
    #[serde(with = "tenth_stops")]
    pub shadow: f64,
    #[serde(with = "tenth_stops")]
    pub color: f64,
    #[serde(with = "tenth_stops")]
    pub sharpness: f64,
    pub clarity: i32,
    pub high_iso_nr: i32,
}

/// Serde for tenth-stop decimals. Whole numbers serialize as integers (no `.0` suffix).
mod tenth_stops {
    use serde::{self, Deserializer, Serializer};

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.fract() == 0.0 {
            serializer.serialize_i64(*value as i64)
        } else {
            serializer.serialize_f64(*value)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = f64;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a number")
            }

            fn visit_f64<E>(self, v: f64) -> Result<f64, E> {
                Ok(v)
            }

            fn visit_i64<E>(self, v: i64) -> Result<f64, E> {
                Ok(v as f64)
            }

            fn visit_u64<E>(self, v: u64) -> Result<f64, E> {
                Ok(v as f64)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Custom serde for exposure in third-stop fractional notation (e.g. `+1/3`, `+2/3`, `-1`).
mod third_stops {
    use serde::{self, Deserializer, Serializer};

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let thirds = (*value * 3.0).round() as i32;
        serializer.serialize_str(&format_thirds(thirds))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StopsVisitor;

        impl<'de> serde::de::Visitor<'de> for StopsVisitor {
            type Value = f64;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a number or fraction like '+1/3'")
            }

            fn visit_f64<E>(self, v: f64) -> Result<f64, E> {
                Ok(v)
            }

            fn visit_i64<E>(self, v: i64) -> Result<f64, E> {
                Ok(v as f64)
            }

            fn visit_u64<E>(self, v: u64) -> Result<f64, E> {
                Ok(v as f64)
            }

            fn visit_str<E>(self, s: &str) -> Result<f64, E>
            where
                E: serde::de::Error,
            {
                parse_fraction(s).ok_or_else(|| E::custom(format!("invalid fractional stops: {s}")))
            }
        }

        deserializer.deserialize_any(StopsVisitor)
    }

    fn format_thirds(thirds: i32) -> String {
        if thirds == 0 {
            return "0".to_string();
        }
        let sign = if thirds > 0 { "+" } else { "-" };
        let abs = thirds.unsigned_abs();
        let whole = abs / 3;
        let rem = abs % 3;
        match (whole, rem) {
            (0, r) => format!("{sign}{r}/3"),
            (w, 0) => format!("{sign}{w}"),
            (w, r) => format!("{sign}{w} {r}/3"),
        }
    }

    fn parse_fraction(s: &str) -> Option<f64> {
        let s = s.trim();
        if s == "0" {
            return Some(0.0);
        }
        let (sign, rest) = if let Some(r) = s.strip_prefix('+') {
            (1.0, r)
        } else if let Some(r) = s.strip_prefix('-') {
            (-1.0, r)
        } else {
            (1.0, s)
        };
        if let Some((num, den)) = rest.split_once('/') {
            let num: f64 = num.trim().parse().ok()?;
            let den: f64 = den.trim().parse().ok()?;
            return Some(sign * num / den);
        }
        if let Some((whole, frac)) = rest.split_once(' ') {
            let whole: f64 = whole.trim().parse().ok()?;
            if let Some((num, den)) = frac.split_once('/') {
                let num: f64 = num.trim().parse().ok()?;
                let den: f64 = den.trim().parse().ok()?;
                return Some(sign * (whole + num / den));
            }
        }
        rest.parse::<f64>().ok().map(|v| sign * v)
    }
}

/// Film simulation modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum FilmSimulation {
    Provia = 1,
    Velvia = 2,
    Astia = 3,
    ProNegHi = 4,
    ProNegStd = 5,
    Monochrome = 6,
    MonochromeY = 7,
    MonochromeR = 8,
    MonochromeG = 9,
    Sepia = 10,
    ClassicChrome = 11,
    Acros = 12,
    AcrosY = 13,
    AcrosR = 14,
    AcrosG = 15,
    Eterna = 16,
    ClassicNeg = 17,
    BleachBypass = 18,
    NostalgicNeg = 19,
    RealaAce = 20,
}

/// Grain effect levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum GrainEffect {
    Off = 1,
    WeakSmall = 2,
    StrongSmall = 3,
    WeakLarge = 4,
    StrongLarge = 5,
}

/// Color Chrome Effect levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum ColorChromeEffect {
    Off = 1,
    Weak = 2,
    Strong = 3,
}

/// D-Range Priority modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, Serialize, Deserialize)]
#[repr(i32)]
pub enum DynamicRangePriority {
    Off = 0,
    Auto = 1,
    Weak = 2,
    Strong = 3,
}

/// Dynamic range modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DynamicRange {
    DR100,
    DR200,
    DR400,
}

/// White balance modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WhiteBalance {
    Auto,
    Daylight,
    Incandescent,
    Underwater,
    Fluorescent1,
    Fluorescent2,
    Fluorescent3,
    Shade,
    Temperature(u32),
}

impl fmt::Display for FilmSimulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provia => write!(f, "Provia"),
            Self::Velvia => write!(f, "Velvia"),
            Self::Astia => write!(f, "Astia"),
            Self::ProNegHi => write!(f, "Pro Neg Hi"),
            Self::ProNegStd => write!(f, "Pro Neg Std"),
            Self::Monochrome => write!(f, "Monochrome"),
            Self::MonochromeY => write!(f, "Monochrome+Y"),
            Self::MonochromeR => write!(f, "Monochrome+R"),
            Self::MonochromeG => write!(f, "Monochrome+G"),
            Self::Sepia => write!(f, "Sepia"),
            Self::ClassicChrome => write!(f, "Classic Chrome"),
            Self::Acros => write!(f, "Acros"),
            Self::AcrosY => write!(f, "Acros+Y"),
            Self::AcrosR => write!(f, "Acros+R"),
            Self::AcrosG => write!(f, "Acros+G"),
            Self::Eterna => write!(f, "Eterna"),
            Self::ClassicNeg => write!(f, "Classic Neg"),
            Self::BleachBypass => write!(f, "Bleach Bypass"),
            Self::NostalgicNeg => write!(f, "Nostalgic Neg"),
            Self::RealaAce => write!(f, "Reala Ace"),
        }
    }
}

impl fmt::Display for GrainEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::WeakSmall => write!(f, "Weak, Small"),
            Self::StrongSmall => write!(f, "Strong, Small"),
            Self::WeakLarge => write!(f, "Weak, Large"),
            Self::StrongLarge => write!(f, "Strong, Large"),
        }
    }
}

impl fmt::Display for ColorChromeEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Weak => write!(f, "Weak"),
            Self::Strong => write!(f, "Strong"),
        }
    }
}

impl fmt::Display for DynamicRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DR100 => write!(f, "DR100"),
            Self::DR200 => write!(f, "DR200"),
            Self::DR400 => write!(f, "DR400"),
        }
    }
}

impl fmt::Display for DynamicRangePriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Off => write!(f, "Off"),
            Self::Auto => write!(f, "Auto"),
            Self::Weak => write!(f, "Weak"),
            Self::Strong => write!(f, "Strong"),
        }
    }
}

impl fmt::Display for WhiteBalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Daylight => write!(f, "Daylight"),
            Self::Incandescent => write!(f, "Incandescent"),
            Self::Underwater => write!(f, "Underwater"),
            Self::Fluorescent1 => write!(f, "Fluorescent 1"),
            Self::Fluorescent2 => write!(f, "Fluorescent 2"),
            Self::Fluorescent3 => write!(f, "Fluorescent 3"),
            Self::Shade => write!(f, "Shade"),
            Self::Temperature(k) => write!(f, "{k}K"),
        }
    }
}
