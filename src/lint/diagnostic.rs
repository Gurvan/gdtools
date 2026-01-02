use std::path::PathBuf;

use crate::lint::Severity;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub end_line: Option<usize>,
    pub end_column: Option<usize>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(rule_id: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            message: message.into(),
            file_path: PathBuf::new(),
            line: 0,
            column: 0,
            end_line: None,
            end_column: None,
            suggestion: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = line;
        self.column = column;
        self
    }

    pub fn with_end_location(mut self, end_line: usize, end_column: usize) -> Self {
        self.end_line = Some(end_line);
        self.end_column = Some(end_column);
        self
    }

    pub fn with_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = path.into();
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let severity_str = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        };
        write!(
            f,
            "{}:{}:{}: {}: {} ({})",
            self.file_path.display(),
            self.line,
            self.column,
            severity_str,
            self.message,
            self.rule_id
        )
    }
}
