// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, Environment, Error, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitPath {
    #[serde(default)]
    style: Style,
}

impl GitPath {
    pub fn new() -> Self {
        GitPath {
            style: Default::default(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        let repo = match environment.repo()? {
            Some(repo) => repo,
            None => return Ok(Vec::new()),
        };
        let path = if repo.is_bare() {
            return Ok(Vec::new());
        } else {
            match repo.path().parent().and_then(|p| p.parent()) {
                Some(p) => match environment.working_dir().strip_prefix(p) {
                    Ok(p) => p,
                    Err(_) => return Ok(Vec::new()),
                }
                None => return Ok(Vec::new()),
            }
        };
        Ok(vec![Block::new(path.to_string_lossy()).with_style(self.style.clone())])
    }
}

impl Default for GitPath {
    fn default() -> Self {
        Self::new()
    }
}
