// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExitCode {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl ExitCode {
    pub fn new() -> Self {
        ExitCode {
            style: Default::default(),
            prefix: default_prefix(),
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

    pub fn with_prefix<T>(self, prefix: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            prefix: prefix.into(),
            ..self
        }
    }

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        match environment.prev_exit_code() {
            0 => Vec::new(),
            code => vec![
                Block::new(&self.prefix).with_style(&self.style),
                Block::new(code.to_string()).with_style(&self.style),
            ],
        }
    }
}

impl Default for ExitCode {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "\u{f071}".into()
}
