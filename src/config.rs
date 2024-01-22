// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, BlockProducer, Environment, Style};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_pretty_prompt")]
    pub prompt: BlockProducer,
    #[serde(default)]
    pub alternative_prompt: Option<BlockProducer>,
    #[serde(with = "humantime_serde", default = "default_timeout")]
    pub timeout: Duration,
}

impl Config {
    pub fn new(prompt: BlockProducer) -> Self {
        Config {
            prompt,
            alternative_prompt: None,
            timeout: default_timeout(),
        }
    }

    pub fn default_pretty() -> Self {
        Config {
            prompt: default_pretty_prompt(),
            alternative_prompt: Some(default_alternative_prompt()),
            timeout: default_timeout(),
        }
    }

    pub fn with_alternative(self, prompt: BlockProducer) -> Self {
        Self {
            alternative_prompt: Some(prompt),
            ..self
        }
    }

    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        let use_alternative = environment.alternative_prompt_is_used();
        let producer = match &self.alternative_prompt {
            Some(p) if use_alternative => p,
            _ => &self.prompt,
        };
        producer.produce(environment)
    }
}

fn default_timeout() -> Duration {
    Duration::from_secs(1)
}

pub fn default_pretty_prompt() -> BlockProducer {
    let id = vec![
        BlockProducer::Username(crate::block::Username::new()),
        BlockProducer::Hostname(crate::block::Hostname::new()),
    ];
    let id = BlockProducer::Separated(crate::block::Separated::new(id).with_separator("@"));
    let path = BlockProducer::Or(crate::block::Or(vec![
        BlockProducer::GitPath(crate::block::GitPath::new()),
        BlockProducer::WorkingDirectory(crate::block::WorkingDirectory::new()),
    ]));
    let info = vec![
        id,
        path,
        BlockProducer::GitHead(crate::block::GitHead::new()),
        BlockProducer::Elapsed(crate::block::Elapsed::new()),
        BlockProducer::ExitCode(crate::block::ExitCode::new().with_style(crate::color::CRIMSON)),
    ];
    let separated = crate::block::Separated::new(info);
    let producer = BlockProducer::Sequence(crate::block::Sequence(vec![
        BlockProducer::Separated(separated),
        BlockProducer::Newline(crate::block::Newline),
        BlockProducer::ExitStatusSymbol(
            crate::block::ExitStatusSymbol::new("→")
                .with_style(crate::color::DODGERBLUE)
                .with_error_style(crate::color::CRIMSON),
        ),
        BlockProducer::Space(crate::block::Space),
    ]));
    BlockProducer::Styled(
        crate::block::Styled::new(producer).with_style(
            Style::new()
                .with_fg(crate::color::TEAL)
                .with_bg(crate::color::BLACK),
        ),
    )
}

pub fn default_alternative_prompt() -> BlockProducer {
    let id = vec![
        BlockProducer::Username(crate::block::Username::new()),
        BlockProducer::Hostname(crate::block::Hostname::new()),
    ];
    let id = BlockProducer::Separated(crate::block::Separated::new(id).with_separator("@"));
    let path =
        BlockProducer::WorkingDirectory(crate::block::WorkingDirectory::new().with_prefix(""));
    let info = vec![
        id,
        path,
        BlockProducer::Elapsed(crate::block::Elapsed::new().with_prefix("")),
        BlockProducer::ExitCode(
            crate::block::ExitCode::new()
                .with_style(crate::color::CRIMSON)
                .with_prefix(""),
        ),
    ];
    let separated = crate::block::Separated::new(info);
    let producer = BlockProducer::Sequence(crate::block::Sequence(vec![
        BlockProducer::Separated(separated),
        BlockProducer::Newline(crate::block::Newline),
        BlockProducer::ExitStatusSymbol(
            crate::block::ExitStatusSymbol::new("→")
                .with_style(crate::color::DODGERBLUE)
                .with_error_style(crate::color::CRIMSON),
        ),
        BlockProducer::Space(crate::block::Space),
    ]));
    BlockProducer::Styled(
        crate::block::Styled::new(producer).with_style(Style::new().with_fg(crate::color::TEAL)),
    )
}

pub fn fallback_prompt() -> BlockProducer {
    BlockProducer::Sequence(crate::block::Sequence(vec![
        BlockProducer::ExitCode(crate::block::ExitCode::new().with_style(crate::color::CRIMSON)),
        BlockProducer::ExitStatusSymbol(
            crate::block::ExitStatusSymbol::new(">")
                .with_style(crate::color::DODGERBLUE)
                .with_error_style(crate::color::CRIMSON),
        ),
        BlockProducer::Space(crate::block::Space),
    ]))
}
