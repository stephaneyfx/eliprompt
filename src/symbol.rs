// Copyright (C) 2020 Stephane Raux. Distributed under the MIT license.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Symbol {
    regular: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback: Option<String>,
}

impl Symbol {
    pub fn new<S: Into<String>>(regular: S) -> Self {
        Self::from(regular.into())
    }

    pub fn with_fallback<S: Into<String>>(self, fallback: S) -> Self {
        Self { fallback: Some(fallback.into()), ..self }
    }

    pub fn as_str(&self, regular: bool) -> &str {
        match (regular, self.fallback.as_ref()) {
            (true, _) | (_, None) => &self.regular,
            (false, Some(fallback)) => fallback,
        }
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self {
            regular: s,
            fallback: None,
        }
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
}
