pub mod config;
pub mod lint;
pub mod parser;
pub mod rules;

pub use lint::{Diagnostic, LintContext, Rule, Severity};
