// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitPath {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl GitPath {
    pub fn new() -> Self {
        GitPath {
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

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        let repo = match environment.repo() {
            Some(repo) => repo,
            None => return Vec::new(),
        };
        let path = if repo.is_bare() {
            return Vec::new();
        } else {
            let Some(p) = repo
                .path()
                .parent()
                .and_then(|p| environment.working_dir()?.strip_prefix(p.parent()?).ok())
            else {
                return Vec::new();
            };
            p
        };
        vec![
            Block::new(&self.prefix).with_style(&self.style),
            Block::new(path.to_string_lossy()).with_style(&self.style),
        ]
    }
}

impl Default for GitPath {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "\u{f7a1}".into()
}
