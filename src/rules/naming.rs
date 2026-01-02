use once_cell::sync::Lazy;
use regex::Regex;
use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{LintContext, Rule, RuleCategory, RuleMetadata, Severity};

static SNAKE_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^_?[a-z][a-z0-9_]*$").unwrap());
static PASCAL_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Z][A-Za-z0-9]*$").unwrap());
static PRIVATE_PASCAL_CASE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^_?[A-Z][A-Za-z0-9]*$").unwrap());
static CONSTANT_CASE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^_?[A-Z][A-Z0-9_]*$").unwrap());
static SIGNAL_HANDLER: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^_on_[A-Za-z0-9]+_[a-z][a-z0-9_]*$").unwrap());
// PascalCase or CONSTANT_CASE (for load constants)
static LOAD_CONSTANT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^_?([A-Z][A-Za-z0-9]*|[A-Z][A-Z0-9_]*)$").unwrap());
// PascalCase or snake_case (for class load variables)
static PASCAL_OR_SNAKE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(_?[A-Z][A-Za-z0-9]*|_?[a-z][a-z0-9_]*)$").unwrap());

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
        // The enumerator node has an identifier child directly
        let name_node = node
            .child_by_field_name("name")
            .or_else(|| node.named_child(0).filter(|c| c.kind() == "identifier"));

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

// ============================================================================
// Additional naming rules
// ============================================================================

#[derive(Debug)]
pub struct FunctionArgumentNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for FunctionArgumentNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "function-argument-name",
                name: "Function Argument Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Function arguments should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for FunctionArgumentNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["parameters"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            let name_node = match child.kind() {
                "identifier" => Some(child),
                "typed_parameter" => child.named_child(0).filter(|c| c.kind() == "identifier"),
                _ => None,
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
                        format!("Function argument \"{}\" should be snake_case", name),
                    );
                }
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoopVariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for LoopVariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "loop-variable-name",
                name: "Loop Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Loop variables should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for LoopVariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["for_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // for_statement has an identifier child for the loop variable
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                let name = ctx.node_text(child);
                if !self.pattern.is_match(name) {
                    let severity = ctx
                        .config()
                        .get_rule_severity(self.meta.id, self.meta.default_severity);
                    ctx.report_node(
                        child,
                        self.meta.id,
                        severity,
                        format!("Loop variable \"{}\" should be snake_case", name),
                    );
                }
                break; // Only check the first identifier (the loop variable)
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct SubClassNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for SubClassNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "sub-class-name",
                name: "Sub Class Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Inner class names should be PascalCase",
            },
            pattern: PRIVATE_PASCAL_CASE.clone(),
        }
    }
}

impl Rule for SubClassNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["class_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Only check inner classes (those with a parent that's not the source)
        if let Some(parent) = node.parent() {
            if parent.kind() != "source" && parent.kind() != "source_file" {
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
                            format!("Inner class name \"{}\" should be PascalCase", name),
                        );
                    }
                }
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoadConstantNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for LoadConstantNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "load-constant-name",
                name: "Load Constant Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Constants with load/preload should be PascalCase or CONSTANT_CASE",
            },
            pattern: LOAD_CONSTANT.clone(),
        }
    }
}

impl Rule for LoadConstantNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["const_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Check if the const value is a load/preload call
        let node_text = ctx.node_text(node);
        if !node_text.contains("load(") && !node_text.contains("preload(") {
            return;
        }

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
                    format!(
                        "Load constant \"{}\" should be PascalCase or CONSTANT_CASE",
                        name
                    ),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// Variable scope-specific naming rules
// ============================================================================

/// Helper to check if a variable_statement is at class scope (not inside a function)
fn is_class_scope_variable(node: Node<'_>) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        match parent.kind() {
            "function_definition" => return false,
            "source_file" | "source" => return true,
            "body" => {
                // Check if this body belongs to a class_definition or function
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == "class_definition" {
                        return true;
                    }
                }
            }
            _ => {}
        }
        current = parent.parent();
    }
    true // Default to class scope if we can't determine
}

/// Helper to check if a variable has a load/preload call
fn has_load_or_preload(node: Node<'_>, ctx: &LintContext<'_>) -> bool {
    let text = ctx.node_text(node);
    text.contains("load(") || text.contains("preload(")
}

#[derive(Debug)]
pub struct ClassVariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for ClassVariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "class-variable-name",
                name: "Class Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Class-scope variables should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for ClassVariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["variable_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Only check class-scope variables without load/preload
        if !is_class_scope_variable(node) || has_load_or_preload(node, ctx) {
            return;
        }

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
                    format!("Class variable \"{}\" should be snake_case", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ClassLoadVariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for ClassLoadVariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "class-load-variable-name",
                name: "Class Load Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Class-scope load/preload variables should be PascalCase or snake_case",
            },
            pattern: PASCAL_OR_SNAKE.clone(),
        }
    }
}

impl Rule for ClassLoadVariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["variable_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Only check class-scope variables with load/preload
        if !is_class_scope_variable(node) || !has_load_or_preload(node, ctx) {
            return;
        }

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
                    format!(
                        "Class load variable \"{}\" should be PascalCase or snake_case",
                        name
                    ),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FunctionVariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for FunctionVariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "function-variable-name",
                name: "Function Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Function-scope variables should be snake_case",
            },
            pattern: SNAKE_CASE.clone(),
        }
    }
}

impl Rule for FunctionVariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["variable_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Only check function-scope variables without load/preload
        if is_class_scope_variable(node) || has_load_or_preload(node, ctx) {
            return;
        }

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
                    format!("Function variable \"{}\" should be snake_case", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FunctionPreloadVariableNameRule {
    meta: RuleMetadata,
    pattern: Regex,
}

impl Default for FunctionPreloadVariableNameRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "function-preload-variable-name",
                name: "Function Preload Variable Name",
                category: RuleCategory::Naming,
                default_severity: Severity::Warning,
                description: "Function-scope preload variables should be PascalCase",
            },
            pattern: PASCAL_CASE.clone(),
        }
    }
}

impl Rule for FunctionPreloadVariableNameRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["variable_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Only check function-scope variables with preload (not load)
        if is_class_scope_variable(node) {
            return;
        }

        let text = ctx.node_text(node);
        if !text.contains("preload(") {
            return;
        }

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
                    format!("Function preload variable \"{}\" should be PascalCase", name),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(pattern) = config.options.get("pattern") {
            if let Some(p) = pattern.as_str() {
                self.pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
            }
        }
        Ok(())
    }
}
