// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Error, Style};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WorkingDirectory {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_home_as_tilde")]
    home_as_tilde: bool,
}

impl WorkingDirectory {
    pub fn new() -> Self {
        WorkingDirectory {
            style: Default::default(),
            home_as_tilde: default_home_as_tilde(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn with_home_as_tilde(self, home_as_tilde: bool) -> Self {
        Self { home_as_tilde, ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        let pwd = environment.working_dir();
        let pwd = if self.home_as_tilde {
            match home_dir() {
                Some(home) => match pwd.strip_prefix(home) {
                    Ok(p) if p.as_os_str().is_empty() => "~".into(),
                    Ok(p) => [Path::new("~"), p].iter().collect(),
                    Err(_) => pwd.to_owned(),
                }
                None => pwd.to_owned(),
            }
        } else {
            pwd.to_owned()
        };
        Ok(vec![Block::new(pwd.to_string_lossy()).with_style(self.style.clone())])
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
