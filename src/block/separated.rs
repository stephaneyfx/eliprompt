// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, BlockProducer, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Separated {
    #[serde(default)]
    separator_style: Style,
    #[serde(default = "default_separator")]
    separator: String,
    producers: Vec<BlockProducer>,
}

impl Separated {
    pub fn new<I>(producers: I) -> Self
    where
        I: IntoIterator<Item = BlockProducer>,
    {
        Self {
            producers: producers.into_iter().collect(),
            ..Default::default()
        }
    }

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self {
            separator_style: style.into(),
            ..self
        }
    }

    pub fn with_separator<T>(self, separator: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            separator: separator.into(),
            ..self
        }
    }

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        self.producers
            .iter()
            .fold(Vec::<Block>::new(), |mut acc, producer| {
                let blocks = producer.produce(environment);
                if !acc.is_empty() && !blocks.is_empty() {
                    acc.push(Block::new(&self.separator).with_style(&self.separator_style));
                }
                acc.extend(blocks);
                acc
            })
    }
}

impl Default for Separated {
    fn default() -> Self {
        Self {
            separator_style: Default::default(),
            separator: default_separator(),
            producers: Default::default(),
        }
    }
}

fn default_separator() -> String {
    " | ".into()
}
