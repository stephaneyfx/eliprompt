// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use crate::{Error, Symbol};
use git2::Repository;
use once_cell::sync::OnceCell;
use std::{
    env,
    fmt::{self, Debug},
    path::{Path, PathBuf},
    time::Duration,
};

pub struct Environment {
    working_dir: PathBuf,
    prev_exit_code: i32,
    repo: OnceCell<Option<Repository>>,
    prev_cmd_duration: Option<Duration>,
    regular_symbols: bool,
}

impl Environment {
    pub fn new<P: Into<PathBuf>>(working_dir: P) -> Result<Self, Error> {
        let term = env::var("TERM").unwrap_or(String::new());
        let regular_symbols = term != "linux";
        Ok(Environment {
            working_dir: working_dir.into(),
            prev_exit_code: 0,
            repo: OnceCell::new(),
            prev_cmd_duration: None,
            regular_symbols,
        })
    }
    pub fn current() -> Result<Self, Error> {
        let working_dir = env::current_dir().map_err(Error::GettingPwdFailed)?;
        Self::new(working_dir)
    }

    pub fn with_prev_exit_code(self, code: i32) -> Self {
        Self { prev_exit_code: code, ..self }
    }

    pub fn with_prev_cmd_duration(self, d: Duration) -> Self {
        Self { prev_cmd_duration: Some(d), ..self }
    }

    pub fn with_regular_symbols(self, regular_symbols: bool) -> Self {
        Self { regular_symbols, ..self }
    }

    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    pub fn repo(&self) -> Result<Option<&Repository>, git2::Error> {
        let mut error = None;
        let repo = self.repo.get_or_init(|| match Repository::discover(&self.working_dir) {
            Ok(repo) => Some(repo),
            Err(e) => {
                error = Some(e);
                None
            }
        });
        match error {
            Some(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Some(e) => Err(e),
            None => Ok(repo.as_ref()),
        }
    }

    pub fn prev_exit_code(&self) -> i32 {
        self.prev_exit_code
    }

    pub fn prev_cmd_duration(&self) -> Option<Duration> {
        self.prev_cmd_duration
    }

    pub fn regular_symbols(&self) -> bool {
        self.regular_symbols
    }

    pub fn symbol_str<'a>(&self, symbol: &'a Symbol) -> &'a str {
        symbol.as_str(self.regular_symbols)
    }
}

impl Debug for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("working_dir", &self.working_dir)
            .field("prev_exit_code", &self.prev_exit_code)
            .field("prev_cmd_duration", &self.prev_cmd_duration)
            .finish()
    }
}
