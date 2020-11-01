// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use rgb::RGB8;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    fmt::{self, Display},
    str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Color {
    inner: RGB8,
    name: Option<Cow<'static, str>>,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Color {
        RGB8::new(r, g, b).into()
    }

    pub fn as_rgb(&self) -> RGB8 {
        self.inner
    }

    const fn named(s: &'static str, value: palette::Srgb<u8>) -> Self {
        Color {
            inner: RGB8 {
                r: value.red,
                g: value.green,
                b: value.blue,
            },
            name: Some(Cow::Borrowed(s)),
        }
    }
}

impl From<RGB8> for Color {
    fn from(c: RGB8) -> Color {
        Color {
            inner: c,
            name: None,
        }
    }
}

impl From<&Color> for Color {
    fn from(c: &Color) -> Color {
        c.clone()
    }
}

impl TryFrom<String> for Color {
    type Error = InvalidColor;

    fn try_from(s: String) -> Result<Color, InvalidColor> {
        let invalid = || InvalidColor(s.clone());
        let (color, name) = if s.starts_with('#') {
            let n = s[1..].parse::<u32>().map_err(|_| invalid())?;
            if n & !0xffffff != 0 { return Err(invalid()) }
            let bytes = n.to_be_bytes();
            (RGB8::from((bytes[1], bytes[2], bytes[3])), None)
        } else {
            let c = palette::named::from_str(&s).ok_or_else(invalid)?;
            (RGB8::from((c.red, c.green, c.blue)), Some(Cow::Owned(s)))
        };
        Ok(Color {
            inner: color,
            name,
        })
    }
}

impl TryFrom<&str> for Color {
    type Error = InvalidColor;

    fn try_from(s: &str) -> Result<Color, InvalidColor> {
        s.to_string().try_into()
    }
}

impl FromStr for Color {
    type Err = InvalidColor;

    fn from_str(s: &str) -> Result<Self, InvalidColor> {
        s.try_into()
    }
}

impl From<palette::Srgb<u8>> for Color {
    fn from(c: palette::Srgb<u8>) -> Color {
        Color::new(c.red, c.green, c.blue)
    }
}

impl From<&Color> for ansi_term::Color {
    fn from(c: &Color) -> Self {
        ansi_term::Color::RGB(c.inner.r, c.inner.g, c.inner.b)
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => f.write_str(name),
            None => write!(f, "#{:02x}{:02x}{:02x}", self.inner.r, self.inner.g, self.inner.b),
        }
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ColorVisitor;

        impl<'v> serde::de::Visitor<'v> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    concat!(
                        r##"a string containing an hexadecimal sRGB color (e.g. "#ff00fe") "##,
                        r##"or a CSS color name"##,
                    ),
                )
            }

            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Color, E> {
                s.parse::<Color>()
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_str(ColorVisitor)
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("Invalid color: {0}")]
pub struct InvalidColor(String);

pub const BLACK: Color = Color::named("black", palette::named::BLACK);
pub const CRIMSON: Color = Color::named("crimson", palette::named::CRIMSON);
pub const CYAN: Color = Color::named("cyan", palette::named::CYAN);
pub const DARKBLUE: Color = Color::named("darkblue", palette::named::DARKBLUE);
pub const DARKCYAN: Color = Color::named("darkcyan", palette::named::DARKCYAN);
pub const DARKGOLDENROD: Color = Color::named("darkgoldenrod", palette::named::DARKGOLDENROD);
pub const DARKGREEN: Color = Color::named("darkgreen", palette::named::DARKGREEN);
pub const DARKRED: Color = Color::named("darkred", palette::named::DARKRED);
pub const DARKSALMON: Color = Color::named("darksalmon", palette::named::DARKSALMON);
pub const DARKSEAGREEN: Color = Color::named("darkseagreen", palette::named::DARKSEAGREEN);
pub const DARKSLATEGRAY: Color = Color::named("darkslategray", palette::named::DARKSLATEGRAY);
pub const DODGERBLUE: Color = Color::named("dodgerblue", palette::named::DODGERBLUE);
pub const FORESTGREEN: Color = Color::named("forestgreen", palette::named::FORESTGREEN);
pub const GOLD: Color = Color::named("gold", palette::named::GOLD);
pub const LIGHTBLUE: Color = Color::named("lightblue", palette::named::LIGHTBLUE);
pub const LIGHTGRAY: Color = Color::named("lightgray", palette::named::LIGHTGRAY);
pub const LIMEGREEN: Color = Color::named("limegreen", palette::named::LIMEGREEN);
pub const MIDNIGHTBLUE: Color = Color::named("midnightblue", palette::named::MIDNIGHTBLUE);
pub const NAVY: Color = Color::named("navy", palette::named::NAVY);
pub const PLUM: Color = Color::named("plum", palette::named::PLUM);
pub const TEAL: Color = Color::named("teal", palette::named::TEAL);
pub const WHITE: Color = Color::named("white", palette::named::WHITE);

#[cfg(test)]
mod tests {
    use crate::Color;
    use rgb::RGB8;

    #[test]
    fn rgb_color_is_printed_as_hex() {
        assert_eq!(Color::from(RGB8::new(255, 0, 0)).to_string(), "#ff0000");
    }
}
