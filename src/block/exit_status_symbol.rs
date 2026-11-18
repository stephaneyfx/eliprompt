// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExitStatusSymbol {
    #[serde(default)]
    style: Style,
    #[serde(default)]
    error_style: Style,
    contents: String,
}

impl ExitStatusSymbol {
    pub fn new<T>(contents: T) -> Self
    where
        T: Into<String>,
    {
        ExitStatusSymbol {
            style: Default::default(),
            error_style: Default::default(),
            contents: contents.into(),
        }
    }

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self {
            style: style.into(),
            ..self
        }
    }

    pub fn with_error_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self {
            error_style: style.into(),
            ..self
        }
    }

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        let style = if environment.prev_exit_code() == 0 {
            &self.style
        } else {
            &self.error_style
        };
        if self.contents.is_empty() {
            Vec::new()
        } else {
            vec![Block::new(&self.contents).with_style(style)]
        }
    }
}
