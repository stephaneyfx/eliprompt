// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Hostname {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl Hostname {
    pub fn new() -> Self {
        Hostname {
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

    pub fn produce(&self, _: &Environment) -> Vec<Block> {
        let hostname = match whoami::hostname() {
            Ok(name) => name,
            Err(e) => {
                tracing::error!("Failed to get hostname: {e}");
                return Vec::new();
            }
        };
        vec![
            Block::new(&self.prefix).with_style(&self.style),
            Block::new(hostname).with_style(&self.style),
        ]
    }
}

impl Default for Hostname {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "".into()
}
