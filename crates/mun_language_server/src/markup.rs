//! Markdown formatting
//!
//! The LSP uses markdown to format "rich text" in UIs.

use std::fmt::{Display, Formatter};

#[derive(Default, Debug)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(m: Markup) -> Self {
        m.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Self { text }
    }
}

impl Display for Markup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}

impl Markup {
    /// Returns the contents as a [`&str`]
    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }
}
