// Copyright (C) 2020 Stephane Raux. Distributed under the 0BSD license.

//! Tools to build a prompt.

#![deny(warnings)]

pub mod block;
pub mod color;
mod config;
mod env;
mod err;
mod style;

pub use block::{Block, BlockProducer};
pub use color::Color;
pub use config::{default_alternative_prompt, default_pretty_prompt, fallback_prompt, Config};
pub use env::Environment;
pub use err::Error;
pub use style::Style;
