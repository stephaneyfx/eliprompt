// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

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
        Default::default()
    }

    pub fn with_fg(self, foreground: Color) -> Style {
        Style { foreground: Some(foreground), ..self }
    }

    pub fn with_bg(self, background: Color) -> Style {
        Style { background: Some(background), ..self }
    }
}
