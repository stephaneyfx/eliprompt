// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get current working directory")]
    GettingPwdFailed(#[source] io::Error),
}
