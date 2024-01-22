// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use git2::Repository;
use once_cell::sync::OnceCell;
use std::{
    env,
    fmt::{self, Debug},
    path::{Path, PathBuf},
    time::Duration,
};

pub struct Environment {
    working_dir: Option<PathBuf>,
    prev_exit_code: i32,
    repo: OnceCell<Option<Repository>>,
    prev_cmd_duration: Option<Duration>,
    force_alternative_prompt: bool,
}

impl Environment {
    pub fn new(working_dir: Option<PathBuf>) -> Self {
        Environment {
            working_dir,
            prev_exit_code: 0,
            repo: OnceCell::new(),
            prev_cmd_duration: None,
            force_alternative_prompt: false,
        }
    }
    pub fn current() -> Self {
        Self::new(env::current_dir().ok())
    }

    pub fn with_prev_exit_code(self, code: i32) -> Self {
        Self {
            prev_exit_code: code,
            ..self
        }
    }

    pub fn with_prev_cmd_duration(self, d: Duration) -> Self {
        Self {
            prev_cmd_duration: Some(d),
            ..self
        }
    }

    pub fn force_alternative_prompt(self, yes: bool) -> Self {
        Self {
            force_alternative_prompt: yes,
            ..self
        }
    }

    pub fn alternative_prompt_is_used(&self) -> bool {
        if self.force_alternative_prompt {
            return true;
        }
        let alternative_requested = env::var("ELIPROMPT_ALTERNATIVE_PROMPT").is_ok();
        let terms_using_alternative = ["linux"];
        let term_uses_alternative =
            env::var("TERM").map_or(false, |term| terms_using_alternative.contains(&&*term));
        alternative_requested || term_uses_alternative
    }

    pub fn working_dir(&self) -> Option<&Path> {
        self.working_dir.as_deref()
    }

    pub fn repo(&self) -> Option<&Repository> {
        let dir = self.working_dir.as_ref()?;
        let repo = self.repo.get_or_init(|| match Repository::discover(dir) {
            Ok(repo) => Some(repo),
            Err(e) if e.code() == git2::ErrorCode::NotFound => None,
            Err(e) => {
                tracing::error!("Failed to open git repository: {}", e);
                None
            }
        });
        repo.as_ref()
    }

    pub fn prev_exit_code(&self) -> i32 {
        self.prev_exit_code
    }

    pub fn prev_cmd_duration(&self) -> Option<Duration> {
        self.prev_cmd_duration
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
