// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use crate::{Block, BlockProducer, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Styled {
    #[serde(default)]
    style: Style,
    producer: Box<BlockProducer>,
}

impl Styled {
    pub fn new(producer: BlockProducer) -> Self {
        Styled {
            style: Default::default(),
            producer: Box::new(producer),
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

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        let mut blocks = self.producer.produce(environment);
        for block in &mut blocks {
            block.style = block.style.or(&self.style);
        }
        blocks
    }
}
