// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, BlockProducer, Environment};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Or(pub Vec<BlockProducer>);

impl Or {
    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        self.0
            .iter()
            .map(|p| p.produce(environment))
            .find(|blocks| !blocks.is_empty())
            .unwrap_or_default()
    }
}
