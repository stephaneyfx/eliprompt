// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use crate::{Block, BlockProducer, Environment};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Sequence(pub Vec<BlockProducer>);

impl Sequence {
    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        self.0.iter().flat_map(|p| p.produce(environment)).collect()
    }
}
