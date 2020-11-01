// Copyright (C) 2020 Stephane Raux. Distributed under the zlib license.

use ansi_term::ANSIString;
use crate::{Environment, Style};
use serde::{Deserialize, Serialize};

mod elapsed;
mod exit_code;
mod exit_status_symbol;
mod git_head;
mod git_path;
mod hostname;
mod newline;
mod or;
mod pwd;
mod separated;
mod sequence;
mod space;
mod styled;
mod text;
mod username;

pub use elapsed::Elapsed;
pub use exit_code::ExitCode;
pub use exit_status_symbol::ExitStatusSymbol;
pub use git_head::GitHead;
pub use git_path::GitPath;
pub use hostname::Hostname;
pub use newline::Newline;
pub use or::Or;
pub use pwd::WorkingDirectory;
pub use separated::Separated;
pub use sequence::Sequence;
pub use space::Space;
pub use styled::Styled;
pub use text::Text;
pub use username::Username;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub text: String,
    pub style: Style,
}

impl Block {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Block {
            text: text.into(),
            style: Default::default(),
        }
    }

    pub fn with_style<T>(self, style: T) -> Self
    where
        T: Into<Style>,
    {
        Block { style: style.into(), ..self }
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
    Hostname(Hostname),
    WorkingDirectory(WorkingDirectory),
    Username(Username),
    Newline(Newline),
    Space(Space),
    Text(Text),
    ExitStatusSymbol(ExitStatusSymbol),
    Or(Or),
    Sequence(Sequence),
    Separated(Separated),
    Styled(Styled),
}

impl BlockProducer {
    pub fn produce(&self, environment: &Environment) -> Vec<Block> {
        match self {
            BlockProducer::Elapsed(p) => p.produce(environment),
            BlockProducer::ExitCode(p) => p.produce(environment),
            BlockProducer::GitHead(p) => p.produce(environment),
            BlockProducer::GitPath(p) => p.produce(environment),
            BlockProducer::Hostname(p) => p.produce(environment),
            BlockProducer::WorkingDirectory(p) => p.produce(environment),
            BlockProducer::Username(p) => p.produce(environment),
            BlockProducer::Newline(p) => p.produce(environment),
            BlockProducer::Space(p) => p.produce(environment),
            BlockProducer::Text(p) => p.produce(environment),
            BlockProducer::ExitStatusSymbol(p) => p.produce(environment),
            BlockProducer::Or(p) => p.produce(environment),
            BlockProducer::Sequence(p) => p.produce(environment),
            BlockProducer::Separated(p) => p.produce(environment),
            BlockProducer::Styled(p) => p.produce(environment),
        }
    }
}
