// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Text {
    #[serde(default)]
    style: Style,
    contents: String,
}

impl Text {
    pub fn new<T>(contents: T) -> Self
    where
        T: Into<String>,
    {
        Text {
            style: Default::default(),
            contents: contents.into(),
        }
    }

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self { style: style.into(), ..self }
    }

    pub fn produce(&self, _: &Environment) -> Vec<Block> {
        vec![Block::new(&self.contents).with_style(&self.style)]
    }
}
