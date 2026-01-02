use std::path::Path;

use tree_sitter::{Node, Tree};

use crate::config::Config;
use crate::lint::{Diagnostic, Severity, Suppressions};

pub struct LintContext<'a> {
    source: &'a str,
    tree: &'a Tree,
    file_path: &'a Path,
    diagnostics: Vec<Diagnostic>,
    suppressions: Suppressions,
    config: &'a Config,
}

impl<'a> LintContext<'a> {
    pub fn new(source: &'a str, tree: &'a Tree, file_path: &'a Path, config: &'a Config) -> Self {
        let suppressions = Suppressions::parse(source);
        Self {
            source,
            tree,
            file_path,
            diagnostics: Vec::new(),
            suppressions,
            config,
        }
    }

    pub fn report(&mut self, diagnostic: Diagnostic) {
        if !self.suppressions.is_suppressed(&diagnostic.rule_id, diagnostic.line) {
            let diag = diagnostic.with_file(self.file_path);
            self.diagnostics.push(diag);
        }
    }

    pub fn report_node(
        &mut self,
        node: Node<'_>,
        rule_id: &str,
        severity: Severity,
        message: impl Into<String>,
    ) {
        let line = node.start_position().row + 1;
        let column = node.start_position().column + 1;
        let end_line = node.end_position().row + 1;
        let end_column = node.end_position().column + 1;

        let diagnostic = Diagnostic::new(rule_id, severity, message)
            .with_location(line, column)
            .with_end_location(end_line, end_column);

        self.report(diagnostic);
    }

    pub fn node_text(&self, node: Node<'_>) -> &str {
        node.utf8_text(self.source.as_bytes()).unwrap_or("")
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn tree(&self) -> &Tree {
        self.tree
    }

    pub fn file_path(&self) -> &Path {
        self.file_path
    }

    pub fn config(&self) -> &Config {
        self.config
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}
