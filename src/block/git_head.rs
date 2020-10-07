// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Error, Style, Symbol};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitHead {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: Symbol,
}

impl GitHead {
    pub fn new() -> Self {
        GitHead {
            style: Default::default(),
            prefix: default_prefix(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn with_prefix<S: Into<Symbol>>(self, prefix: S) -> Self {
        Self { prefix: prefix.into(), ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        let repo = match environment.repo()? {
            Some(repo) => repo,
            None => return Ok(Vec::new()),
        };
        let head = repo.head();
        let name = match head {
            Ok(ref head) => match head.shorthand() {
                Some(s) => s,
                None => return Ok(Vec::new()),
            }
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => "master".into(),
            Err(e) => return Err(e.into()),
        };
        Ok(vec![
            Block::new(environment.symbol_str(&self.prefix)).with_style(self.style.clone()),
            Block::new(name).with_style(self.style.clone()),
        ])
    }
}

impl Default for GitHead {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> Symbol {
    Symbol::from("\u{e725}").with_fallback("[git]")
}
