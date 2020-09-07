// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

//! Tools to build a prompt.

#![deny(warnings)]

pub mod block;
mod color;
mod config;
mod env;
mod err;
mod style;
mod symbol;

pub use block::{Block, BlockProducer};
pub use color::Color;
pub use config::Config;
pub use env::Environment;
pub use err::Error;
pub use style::Style;
pub use symbol::Symbol;
