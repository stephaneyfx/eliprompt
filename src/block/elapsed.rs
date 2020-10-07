// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Error, Style, Symbol};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Elapsed {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: Symbol,
    #[serde(with = "humantime_serde", default = "default_threshold")]
    threshold: Duration,
}

impl Elapsed {
    pub fn new() -> Self {
        Elapsed {
            style: Default::default(),
            prefix: default_prefix(),
            threshold: default_threshold(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn with_prefix<S: Into<Symbol>>(self, prefix: S) -> Self {
        Self { prefix: prefix.into(), ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        match environment.prev_cmd_duration() {
            Some(elapsed) if elapsed >= self.threshold => {
                let elapsed = Duration::from_secs(elapsed.as_secs())
                    + Duration::from_millis(elapsed.subsec_millis() as u64);
                let prefix = environment.symbol_str(&self.prefix);
                let elapsed = humantime::format_duration(elapsed).to_string();
                Ok(vec![
                    Block::new(prefix).with_style(self.style.clone()),
                    Block::new(elapsed).with_style(self.style.clone()),
                ])
            }
            _ => Ok(Vec::new()),
        }
    }
}

impl Default for Elapsed {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> Symbol {
    Symbol::from("\u{fa1a}").with_fallback("[took]")
}

fn default_threshold() -> Duration {
    Duration::from_secs(2)
}
