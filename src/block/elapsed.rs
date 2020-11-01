// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, Environment, Style};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Elapsed {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: String,
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

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Self { style: style.into(), ..self }
    }

    pub fn with_prefix<T>(self, prefix: T) -> Self
    where
        T: Into<String>,
    {
        Self { prefix: prefix.into(), ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        match environment.prev_cmd_duration() {
            Some(elapsed) if elapsed >= self.threshold => {
                let elapsed = Duration::from_secs(elapsed.as_secs())
                    + Duration::from_millis(elapsed.subsec_millis() as u64);
                let elapsed = humantime::format_duration(elapsed).to_string();
                vec![
                    Block::new(&self.prefix).with_style(&self.style),
                    Block::new(elapsed).with_style(&self.style),
                ]
            }
            Some(_) => Vec::new(),
            None => {
                tracing::warn!("Previous command duration unavailable");
                Vec::new()
            }
        }
    }
}

impl Default for Elapsed {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> String {
    "\u{fa1a}".into()
}

fn default_threshold() -> Duration {
    Duration::from_secs(2)
}
