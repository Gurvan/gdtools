pub mod config;
pub mod lint;
pub mod parser;
pub mod rules;

pub use lint::{run_linter, Diagnostic, LintContext, Rule, Severity};
