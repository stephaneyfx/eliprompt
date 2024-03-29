// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, Environment, Style};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkingDirectory {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_home_as_tilde")]
    home_as_tilde: bool,
    #[serde(default = "default_prefix")]
    prefix: String,
}

impl WorkingDirectory {
    pub fn new() -> Self {
        WorkingDirectory {
            style: Default::default(),
            home_as_tilde: default_home_as_tilde(),
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

    pub fn with_home_as_tilde(self, home_as_tilde: bool) -> Self {
        Self {
            home_as_tilde,
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
        let pwd = match environment.working_dir() {
            Some(pwd) if self.home_as_tilde => {
                match home_dir().and_then(|home| pwd.strip_prefix(home).ok()) {
                    Some(p) if p.as_os_str().is_empty() => "~".into(),
                    Some(p) => [Path::new("~"), p].iter().collect(),
                    None => pwd.to_owned(),
                }
            }
            Some(pwd) => pwd.to_owned(),
            None => "<NONE>".into(),
        };
        let pwd = pwd.to_string_lossy();
        vec![
            Block::new(&self.prefix).with_style(&self.style),
            Block::new(pwd).with_style(&self.style),
        ]
    }
}

impl Default for WorkingDirectory {
    fn default() -> Self {
        Self::new()
    }
}

fn default_home_as_tilde() -> bool {
    true
}

fn default_prefix() -> String {
    "\u{f07c}".into()
}
