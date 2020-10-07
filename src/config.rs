// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use crate::{Block, BlockProducer, Environment, Error, Style, Symbol};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    block_producers: Vec<BlockProducer>,
    #[serde(default = "default_prompt")]
    prompt: Symbol,
    #[serde(default)]
    prompt_style: Style,
    #[serde(default = "default_prompt_error_style")]
    prompt_error_style: Style,
    #[serde(default = "default_separator")]
    separator: Symbol,
    #[serde(default)]
    separator_style: Style,
    #[serde(with = "humantime_serde", default = "default_timeout")]
    timeout: Duration,
}

impl Config {
    pub fn new() -> Self {
        Config {
            block_producers: Vec::new(),
            prompt: default_prompt(),
            prompt_style: Default::default(),
            prompt_error_style: default_prompt_error_style(),
            separator: default_separator(),
            separator_style: Default::default(),
            timeout: default_timeout(),
        }
    }

    pub fn from_producers<P>(producers: P) -> Self
    where
        P: IntoIterator<Item = BlockProducer>,
    {
        Config { block_producers: producers.into_iter().collect(), ..Self::new() }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    pub fn append_producer(mut self, p: BlockProducer) -> Self {
        self.block_producers.push(p);
        self
    }

    pub fn with_prompt<S: Into<Symbol>>(self, prompt: S) -> Self {
        Self { prompt: prompt.into(), ..self }
    }

    pub fn with_prompt_style(self, prompt_style: Style) -> Self {
        Self { prompt_style, ..self }
    }

    pub fn with_prompt_error_style(self, prompt_error_style: Style) -> Self {
        Self { prompt_error_style, ..self }
    }

    pub fn with_separator<S: Into<Symbol>>(self, separator: S) -> Self {
        Self { separator: separator.into(), ..self }
    }

    pub fn with_separator_style(self, separator_style: Style) -> Self {
        Self { separator_style, ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        let prompt_block = Block::new(environment.symbol_str(&self.prompt));
        let prompt_block = match environment.prev_exit_code() {
            0 => prompt_block.with_style(self.prompt_style.clone()),
            _ => prompt_block.with_style(self.prompt_error_style.clone()),
        };
        let separator = environment.symbol_str(&self.separator);
        let separator = Block::new(separator).with_style(self.separator_style.clone());
        self
            .block_producers
            .iter()
            .map(|p| p.produce(environment))
            .try_fold((Vec::new(), false), |(mut acc, separator_at_end), blocks| {
                let blocks = blocks?;
                let has_blocks = !blocks.is_empty();
                acc.extend(blocks);
                if has_blocks {
                    acc.push(separator.clone());
                }
                Ok((acc, has_blocks || separator_at_end))
            })
            .map(|(mut blocks, separator_at_end)| {
                if separator_at_end {
                    blocks.pop();
                }
                blocks.push(Block::new("\n"));
                blocks.push(prompt_block);
                blocks.push(Block::new(" "));
                blocks
            })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prompt() -> Symbol {
    Symbol::from("\u{2192}").with_fallback(">")
}

fn default_prompt_error_style() -> Style {
    Style::new().with_fg("red".parse().unwrap())
}

fn default_separator() -> Symbol {
    " ".into()
}

fn default_timeout() -> Duration {
    Duration::from_secs(1)
}
