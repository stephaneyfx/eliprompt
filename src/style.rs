// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Style {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Color>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg<T>(foreground: T) -> Self
    where
        T: Into<Color>,
    {
        Self {
            foreground: Some(foreground.into()),
            background: None,
        }
    }

    pub fn bg<T>(background: T) -> Self
    where
        T: Into<Color>,
    {
        Self {
            background: Some(background.into()),
            foreground: None,
        }
    }

    pub fn with_fg<T>(self, foreground: T) -> Style
    where
        T: Into<Color>,
    {
        Style {
            foreground: Some(foreground.into()),
            ..self
        }
    }

    pub fn with_bg<T>(self, background: T) -> Style
    where
        T: Into<Color>,
    {
        Style {
            background: Some(background.into()),
            ..self
        }
    }

    pub fn with_maybe_fg(self, foreground: Option<Color>) -> Style {
        Style { foreground, ..self }
    }

    pub fn with_maybe_bg(self, background: Option<Color>) -> Style {
        Style { background, ..self }
    }

    pub fn or(&self, default: &Style) -> Style {
        Style {
            foreground: self
                .foreground
                .clone()
                .or_else(|| default.foreground.clone()),
            background: self
                .background
                .clone()
                .or_else(|| default.background.clone()),
        }
    }
}

impl From<Color> for Style {
    fn from(c: Color) -> Style {
        Style::fg(c)
    }
}

impl From<&Color> for Style {
    fn from(c: &Color) -> Style {
        c.clone().into()
    }
}

impl From<&Style> for Style {
    fn from(s: &Style) -> Style {
        s.clone()
    }
}
