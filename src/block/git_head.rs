// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitHead {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl GitHead {
    pub fn new() -> Self {
        GitHead {
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
        let head = repo.head();
        let name = match head {
            Ok(ref head) => match head.shorthand() {
                Some(s) => s,
                None => return Vec::new(),
            },
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => "master",
            Err(e) => {
                tracing::error!("Failed to get git repository HEAD: {}", e);
                return Vec::new();
            }
        };
        vec![
            Block::new(&self.prefix).with_style(&self.style),
            Block::new(name).with_style(&self.style),
        ]
    }
}

impl Default for GitHead {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "\u{e725}".into()
}
