// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Space;

impl Space {
    pub fn produce(&self, _: &Environment) -> Vec<Block> {
        vec![Block::new(" ")]
    }
}
