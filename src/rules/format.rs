use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{Diagnostic, LintContext, Rule, RuleCategory, RuleMetadata, Severity};

#[derive(Debug)]
pub struct MaxLineLengthRule {
    meta: RuleMetadata,
    max_length: usize,
    tab_width: usize,
}

impl Default for MaxLineLengthRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "max-line-length",
                name: "Maximum Line Length",
                category: RuleCategory::Format,
                default_severity: Severity::Warning,
                description: "Lines should not exceed the maximum length",
            },
            max_length: 100,
            tab_width: 4,
        }
    }
}

impl Rule for MaxLineLengthRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        None
    }

    fn check_node(&self, _node: Node<'_>, _ctx: &mut LintContext<'_>) {}

    fn check_file_start(&self, ctx: &mut LintContext<'_>) {
        let severity = ctx
            .config()
            .get_rule_severity(self.meta.id, self.meta.default_severity);

        let source = ctx.source().to_string();
        let mut diagnostics = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let visual_length = self.calculate_visual_length(line);

            if visual_length > self.max_length {
                let line_num = line_idx + 1;
                let diagnostic = Diagnostic::new(
                    self.meta.id,
                    severity,
                    format!(
                        "Line is {} characters long (max {})",
                        visual_length, self.max_length
                    ),
                )
                .with_location(line_num, self.max_length + 1);

                diagnostics.push(diagnostic);
            }
        }

        for diagnostic in diagnostics {
            ctx.report(diagnostic);
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(max) = config.options.get("max") {
            if let Some(n) = max.as_integer() {
                self.max_length = n as usize;
            }
        }
        if let Some(max) = config.options.get("max_length") {
            if let Some(n) = max.as_integer() {
                self.max_length = n as usize;
            }
        }
        if let Some(tab) = config.options.get("tab_width") {
            if let Some(n) = tab.as_integer() {
                self.tab_width = n as usize;
            }
        }
        Ok(())
    }
}

impl MaxLineLengthRule {
    fn calculate_visual_length(&self, line: &str) -> usize {
        let mut length = 0;
        for c in line.chars() {
            if c == '\t' {
                length += self.tab_width - (length % self.tab_width);
            } else {
                length += 1;
            }
        }
        length
    }
}

#[derive(Debug)]
pub struct TrailingWhitespaceRule {
    meta: RuleMetadata,
}

impl Default for TrailingWhitespaceRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "trailing-whitespace",
                name: "Trailing Whitespace",
                category: RuleCategory::Format,
                default_severity: Severity::Warning,
                description: "Lines should not have trailing whitespace",
            },
        }
    }
}

impl Rule for TrailingWhitespaceRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        None
    }

    fn check_node(&self, _node: Node<'_>, _ctx: &mut LintContext<'_>) {}

    fn check_file_start(&self, ctx: &mut LintContext<'_>) {
        let severity = ctx
            .config()
            .get_rule_severity(self.meta.id, self.meta.default_severity);

        let source = ctx.source().to_string();
        let mut diagnostics = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            if line.ends_with(' ') || line.ends_with('\t') {
                let trimmed_len = line.trim_end().len();
                let line_num = line_idx + 1;
                let diagnostic = Diagnostic::new(self.meta.id, severity, "Trailing whitespace")
                    .with_location(line_num, trimmed_len + 1);

                diagnostics.push(diagnostic);
            }
        }

        for diagnostic in diagnostics {
            ctx.report(diagnostic);
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MixedTabsSpacesRule {
    meta: RuleMetadata,
}

impl Default for MixedTabsSpacesRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "mixed-tabs-spaces",
                name: "Mixed Tabs and Spaces",
                category: RuleCategory::Format,
                default_severity: Severity::Warning,
                description: "Indentation should not mix tabs and spaces",
            },
        }
    }
}

impl Rule for MixedTabsSpacesRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        None
    }

    fn check_node(&self, _node: Node<'_>, _ctx: &mut LintContext<'_>) {}

    fn check_file_start(&self, ctx: &mut LintContext<'_>) {
        let severity = ctx
            .config()
            .get_rule_severity(self.meta.id, self.meta.default_severity);

        let source = ctx.source().to_string();
        let mut diagnostics = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();

            if indent.contains('\t') && indent.contains(' ') {
                let line_num = line_idx + 1;
                let diagnostic = Diagnostic::new(
                    self.meta.id,
                    severity,
                    "Mixed tabs and spaces in indentation",
                )
                .with_location(line_num, 1);

                diagnostics.push(diagnostic);
            }
        }

        for diagnostic in diagnostics {
            ctx.report(diagnostic);
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MaxFileLinesRule {
    meta: RuleMetadata,
    max_lines: usize,
}

impl Default for MaxFileLinesRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "max-file-lines",
                name: "Maximum File Lines",
                category: RuleCategory::Format,
                default_severity: Severity::Warning,
                description: "Files should not exceed the maximum number of lines",
            },
            max_lines: 1000,
        }
    }
}

impl Rule for MaxFileLinesRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        None
    }

    fn check_node(&self, _node: Node<'_>, _ctx: &mut LintContext<'_>) {}

    fn check_file_start(&self, ctx: &mut LintContext<'_>) {
        let line_count = ctx.source().lines().count();

        if line_count > self.max_lines {
            let severity = ctx
                .config()
                .get_rule_severity(self.meta.id, self.meta.default_severity);
            let diagnostic = Diagnostic::new(
                self.meta.id,
                severity,
                format!("File has {} lines (max {})", line_count, self.max_lines),
            )
            .with_location(self.max_lines + 1, 1);

            ctx.report(diagnostic);
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(max) = config.options.get("max") {
            if let Some(n) = max.as_integer() {
                self.max_lines = n as usize;
            }
        }
        if let Some(max) = config.options.get("max_lines") {
            if let Some(n) = max.as_integer() {
                self.max_lines = n as usize;
            }
        }
        Ok(())
    }
}
