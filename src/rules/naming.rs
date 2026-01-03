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

/// Helper to configure a pattern from rule config.
fn configure_pattern(pattern: &mut Regex, config: &RuleConfig) -> Result<(), String> {
    if let Some(p) = config.options.get("pattern").and_then(|v| v.as_str()) {
        *pattern = Regex::new(p).map_err(|e| format!("Invalid pattern: {}", e))?;
    }
    Ok(())
}

/// Macro to generate simple naming rules that check a "name" field against a pattern.
macro_rules! simple_naming_rule {
    (
        $struct_name:ident,
        $id:literal,
        $name:literal,
        $description:literal,
        $default_pattern:expr,
        $node_kinds:expr,
        $message:literal
    ) => {
        #[derive(Debug)]
        pub struct $struct_name {
            meta: RuleMetadata,
            pattern: Regex,
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    meta: RuleMetadata {
                        id: $id,
                        name: $name,
                        category: RuleCategory::Naming,
                        default_severity: Severity::Warning,
                        description: $description,
                    },
                    pattern: $default_pattern.clone(),
                }
            }
        }

        impl Rule for $struct_name {
            fn meta(&self) -> &RuleMetadata {
                &self.meta
            }

            fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
                Some($node_kinds)
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
                            format!(concat!($message, " \"{}\""), name),
                        );
                    }
                }
            }

            fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
                configure_pattern(&mut self.pattern, config)
            }
        }
    };
}

// Generate simple naming rules
simple_naming_rule!(
    ClassNameRule,
    "class-name",
    "Class Name",
    "Class names should be PascalCase",
    PASCAL_CASE,
    &["class_name_statement", "class_definition"],
    "Class name should be PascalCase:"
);

simple_naming_rule!(
    SignalNameRule,
    "signal-name",
    "Signal Name",
    "Signal names should be snake_case",
    SNAKE_CASE,
    &["signal_statement"],
    "Signal name should be snake_case:"
);

simple_naming_rule!(
    ConstantNameRule,
    "constant-name",
    "Constant Name",
    "Constants should be CONSTANT_CASE",
    CONSTANT_CASE,
    &["const_statement"],
    "Constant name should be CONSTANT_CASE:"
);

simple_naming_rule!(
    EnumNameRule,
    "enum-name",
    "Enum Name",
    "Enum names should be PascalCase",
    PASCAL_CASE,
    &["enum_definition"],
    "Enum name should be PascalCase:"
);

// ============================================================================
// Rules that need custom check_node logic
// ============================================================================

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
            // Allow signal handler pattern: _on_NodeName_signal_name
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
        configure_pattern(&mut self.pattern, config)
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
        configure_pattern(&mut self.pattern, config)
    }
}

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
        configure_pattern(&mut self.pattern, config)
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
        configure_pattern(&mut self.pattern, config)
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
        configure_pattern(&mut self.pattern, config)
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
        configure_pattern(&mut self.pattern, config)
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

/// Macro to generate variable naming rules with scope filtering.
macro_rules! variable_naming_rule {
    (
        $struct_name:ident,
        $id:literal,
        $name:literal,
        $description:literal,
        $default_pattern:expr,
        $scope_filter:expr,
        $message:literal
    ) => {
        #[derive(Debug)]
        pub struct $struct_name {
            meta: RuleMetadata,
            pattern: Regex,
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    meta: RuleMetadata {
                        id: $id,
                        name: $name,
                        category: RuleCategory::Naming,
                        default_severity: Severity::Warning,
                        description: $description,
                    },
                    pattern: $default_pattern.clone(),
                }
            }
        }

        impl Rule for $struct_name {
            fn meta(&self) -> &RuleMetadata {
                &self.meta
            }

            fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
                Some(&["variable_statement"])
            }

            fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
                let filter: fn(Node<'_>, &LintContext<'_>) -> bool = $scope_filter;
                if !filter(node, ctx) {
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
                            format!(concat!($message, " \"{}\""), name),
                        );
                    }
                }
            }

            fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
                configure_pattern(&mut self.pattern, config)
            }
        }
    };
}

variable_naming_rule!(
    ClassVariableNameRule,
    "class-variable-name",
    "Class Variable Name",
    "Class-scope variables should be snake_case",
    SNAKE_CASE,
    |node, ctx| is_class_scope_variable(node) && !has_load_or_preload(node, ctx),
    "Class variable should be snake_case:"
);

variable_naming_rule!(
    ClassLoadVariableNameRule,
    "class-load-variable-name",
    "Class Load Variable Name",
    "Class-scope load/preload variables should be PascalCase or snake_case",
    PASCAL_OR_SNAKE,
    |node, ctx| is_class_scope_variable(node) && has_load_or_preload(node, ctx),
    "Class load variable should be PascalCase or snake_case:"
);

variable_naming_rule!(
    FunctionVariableNameRule,
    "function-variable-name",
    "Function Variable Name",
    "Function-scope variables should be snake_case",
    SNAKE_CASE,
    |node, ctx| !is_class_scope_variable(node) && !has_load_or_preload(node, ctx),
    "Function variable should be snake_case:"
);

variable_naming_rule!(
    FunctionPreloadVariableNameRule,
    "function-preload-variable-name",
    "Function Preload Variable Name",
    "Function-scope preload variables should be PascalCase",
    PASCAL_CASE,
    |node, ctx| {
        if is_class_scope_variable(node) {
            return false;
        }
        let text = ctx.node_text(node);
        text.contains("preload(")
    },
    "Function preload variable should be PascalCase:"
);
