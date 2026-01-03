pub mod config;
pub mod format;
pub mod lint;
pub mod parser;
pub mod rules;

pub use format::{run_formatter, FormatError, FormatOptions, IndentStyle};
pub use lint::{run_linter, Diagnostic, LintContext, Rule, Severity};
