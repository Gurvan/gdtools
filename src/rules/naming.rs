use once_cell::sync::Lazy;
use regex::Regex;
use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{LintContext, Rule, RuleCategory, RuleMetadata, Severity};

static SNAKE_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^_?[a-z][a-z0-9_]*$").unwrap());
static PASCAL_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z][A-Za-z0-9]*$").unwrap());
static CONSTANT_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^_?[A-Z][A-Z0-9_]*$").unwrap());
static SIGNAL_HANDLER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^_on_[A-Za-z0-9]+_[a-z][a-z0-9_]*$").unwrap());

#[derive(Debug)]
pub struct FunctionNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for FunctionNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "function-name",
                name: "Function Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Function names should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for FunctionNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["function_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) && !SIGNAL_HANDLER.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Function name \"{}\" should be snake_case", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClassNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for ClassNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "class-name",
                name: "Class Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Class names should be PascalCase",
            },
            pattern: PASCAL_CASE.clone(),
        }
    }
}

impl Rule for ClassNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["class_name_statement", "class_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let name_node = if node.kind() == "class_name_statement" {
            node.child_by_field_name("name")
        } else {
            node.child_by_field_name("name")
        };

        if let Some(name_node) = name_node {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Class name \"{}\" should be PascalCase", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SignalNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for SignalNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "signal-name",
                name: "Signal Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Signal names should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for SignalNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["signal_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Signal name \"{}\" should be snake_case", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConstantNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for ConstantNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "constant-name",
                name: "Constant Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Constants should be CONSTANT_CASE",
            },
            pattern: CONSTANT_CASE.clone(),
        }
    }
}

impl Rule for ConstantNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["const_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Constant name \"{}\" should be CONSTANT_CASE", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct VariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for VariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "variable-name",
                name: "Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Variables should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for VariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["variable_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Variable name \"{}\" should be snake_case", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct EnumNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for EnumNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "enum-name",
                name: "Enum Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Enum names should be PascalCase",
            },
            pattern: PASCAL_CASE.clone(),
        }
    }
}

impl Rule for EnumNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["enum_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Enum name \"{}\" should be PascalCase", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct EnumElementNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for EnumElementNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "enum-element-name",
                name: "Enum Element Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Enum elements should be CONSTANT_CASE",
            },
            pattern: CONSTANT_CASE.clone(),
        }
    }
}

impl Rule for EnumElementNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["enumerator"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = ctx.node_text(name_node);

            if !self.pattern.is_match(name) {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Enum element \"{}\" should be CONSTANT_CASE", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern =
                    Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}
