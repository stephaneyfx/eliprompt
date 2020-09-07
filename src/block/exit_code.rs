// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Block, Environment, Error, Style, Symbol};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExitCode {
    #[serde(default)]
    style: Style,
    #[serde(default = "default_prefix")]
    prefix: Symbol,
}

impl ExitCode {
    pub fn new() -> Self {
        ExitCode {
            style: Default::default(),
            prefix: default_prefix(),
        }
    }

    pub fn with_style(self, style: Style) -> Self {
        Self { style, ..self }
    }

    pub fn with_prefix<S: Into<Symbol>>(self, prefix: S) -> Self {
        Self { prefix: prefix.into(), ..self }
    }

    pub fn produce(&self, environment: &Environment) -> Result<Vec<Block>, Error> {
        match environment.prev_exit_code() {
            0 => Ok(Vec::new()),
            code => Ok(vec![
                Block::new(environment.symbol_str(&self.prefix)).with_style(self.style.clone()),
                Block::new(code.to_string()).with_style(self.style.clone()),
            ]),
        }
    }
}

impl Default for ExitCode {
    fn default() -> Self {
        Self::new()
    }
}

fn default_prefix() -> Symbol {
    Symbol::from("\u{274c}").with_fallback("X")
}
