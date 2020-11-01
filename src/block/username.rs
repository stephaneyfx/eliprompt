// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Username {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl Username {
    pub fn new() -> Self {
        Username {
            style: Default::default(),
            prefix: default_prefix(),
        }
    }

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self { style: style.into(), ..self }
    }

    pub fn with_prefix<T>(self, prefix: T) -> Self
    where
        T: Into<String>,
    {
        Self { prefix: prefix.into(), ..self }
    }

    pub fn produce(&self, _: &Environment) -> Vec<Block> {
        vec![
            Block::new(&self.prefix).with_style(&self.style),
            Block::new(whoami::username()).with_style(&self.style),
        ]
    }
}

impl Default for Username {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "".into()
}
