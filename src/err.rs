// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get current working directory")]
    GettingPwdFailed(#[source] io::Error),
    #[error("Git error")]
    Git(#[from] git2::Error),
}
