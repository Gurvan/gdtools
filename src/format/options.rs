use serde::{Deserialize, Serialize};

/// Indentation style for formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IndentStyle {
    Tabs,
    Spaces(usize),
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Tabs
    }
}

impl IndentStyle {
    /// Get the string representation of one indent level.
    pub fn as_str(&self) -> String {
        match self {
            IndentStyle::Tabs => "\t".to_string(),
            IndentStyle::Spaces(n) => " ".repeat(*n),
        }
    }

    /// Get the visual width of one indent level (for line length calculation).
    pub fn width(&self) -> usize {
        match self {
            IndentStyle::Tabs => 4, // Tab counts as 4 spaces for line length
            IndentStyle::Spaces(n) => *n,
        }
    }
}

/// Formatting options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOptions {
    /// Indentation style (tabs or spaces).
    #[serde(default)]
    pub indent_style: IndentStyle,

    /// Maximum line length before breaking.
    #[serde(default = "default_line_length")]
    pub max_line_length: usize,

    /// Whether to ensure a trailing newline at end of file.
    #[serde(default = "default_true")]
    pub trailing_newline: bool,
}

fn default_line_length() -> usize {
    100
}

fn default_true() -> bool {
    true
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            max_line_length: default_line_length(),
            trailing_newline: true,
        }
    }
}

impl FormatOptions {
    /// Create options with tabs indentation.
    pub fn with_tabs() -> Self {
        Self {
            indent_style: IndentStyle::Tabs,
            ..Default::default()
        }
    }

    /// Create options with spaces indentation.
    pub fn with_spaces(n: usize) -> Self {
        Self {
            indent_style: IndentStyle::Spaces(n),
            ..Default::default()
        }
    }

    /// Set the maximum line length.
    pub fn line_length(mut self, len: usize) -> Self {
        self.max_line_length = len;
        self
    }
}
