// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use ansi_term::ANSIString;
use crate::{Environment, Error, Style};
use serde::{Deserialize, Serialize};

mod elapsed;
mod exit_code;
mod git_head;
mod git_path;
mod pwd;

pub use elapsed::Elapsed;
pub use exit_code::ExitCode;
pub use git_head::GitHead;
pub use git_path::GitPath;
pub use pwd::WorkingDirectory;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub text: String,
    pub style: Style,
}

impl Block {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Block {
            text: text.into(),
            style: Default::default(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Block { style, ..self }
    }

    pub fn render(&self) -> ANSIString<'_> {
        let style = ansi_term::Style::new();
        let style = match &self.style.foreground {
            Some(fg) => style.fg(fg.into()),
            None => style,
        };
        let style = match &self.style.background {
            Some(bg) => style.on(bg.into()),
            None => style,
        };
        style.paint(&self.text)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum BlockProducer {
    Elapsed(Elapsed),
    ExitCode(ExitCode),
    GitHead(GitHead),
    GitPath(GitPath),
    Or(Vec<BlockProducer>),
    WorkingDirectory(WorkingDirectory),
}

impl BlockProducer {
    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        match self {
            BlockProducer::Elapsed(p) => p.produce(environment),
            BlockProducer::ExitCode(p) => p.produce(environment),
            BlockProducer::GitHead(p) => p.produce(environment),
            BlockProducer::GitPath(p) => p.produce(environment),
            BlockProducer::Or(producers) => {
                producers
                    .iter()
                    .map(|p| p.produce(environment))
                    .find(|blocks| blocks.as_ref().map_or(true, |blocks| !blocks.is_empty()))
                    .unwrap_or_else(|| Ok(Vec::new()))
            }
            BlockProducer::WorkingDirectory(p) => p.produce(environment),
        }
    }
}
