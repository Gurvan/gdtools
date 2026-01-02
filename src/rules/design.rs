use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{LintContext, Rule, RuleCategory, RuleMetadata, Severity};

#[derive(Debug)]
pub struct MaxFunctionArgsRule {
    meta: RuleMetadata,
    max_args: usize,
}

impl Default for MaxFunctionArgsRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "max-function-args",
                name: "Maximum Function Arguments",
                category: RuleCategory::Design,
                default_severity: Severity::Warning,
                description: "Functions should not have too many arguments",
            },
            max_args: 10,
        }
    }
}

impl Rule for MaxFunctionArgsRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["function_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(params_node) = node.child_by_field_name("parameters") {
            let mut arg_count = 0;
            let mut cursor = params_node.walk();
            for child in params_node.children(&mut cursor) {
                match child.kind() {
                    "identifier" | "typed_parameter" | "default_parameter" => {
                        arg_count += 1;
                    }
                    _ => {}
                }
            }

            if arg_count > self.max_args {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);

                let func_name = node
                    .child_by_field_name("name")
                    .map(|n| ctx.node_text(n))
                    .unwrap_or("<anonymous>");

                ctx.report_node(
                    node,
                    self.meta.id,
                    severity,
                    format!(
                        "Function \"{}\" has {} arguments (max {})",
                        func_name, arg_count, self.max_args
                    ),
                );
            }
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(max) = config.options.get("max") {
            if let Some(n) = max.as_integer() {
                self.max_args = n as usize;
            }
        }
        if let Some(max) = config.options.get("max_args") {
            if let Some(n) = max.as_integer() {
                self.max_args = n as usize;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct MaxReturnsRule {
    meta: RuleMetadata,
    max_returns: usize,
}

impl Default for MaxReturnsRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "max-returns",
                name: "Maximum Return Statements",
                category: RuleCategory::Design,
                default_severity: Severity::Warning,
                description: "Functions should not have too many return statements",
            },
            max_returns: 6,
        }
    }
}

impl Rule for MaxReturnsRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["function_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let return_count = count_returns(node);

        if return_count > self.max_returns {
            let severity = ctx
                .config()
                .get_rule_severity(self.meta.id, self.meta.default_severity);

            let func_name = node
                .child_by_field_name("name")
                .map(|n| ctx.node_text(n))
                .unwrap_or("<anonymous>");

            ctx.report_node(
                node,
                self.meta.id,
                severity,
                format!(
                    "Function \"{}\" has {} return statements (max {})",
                    func_name, return_count, self.max_returns
                ),
            );
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(max) = config.options.get("max") {
            if let Some(n) = max.as_integer() {
                self.max_returns = n as usize;
            }
        }
        if let Some(max) = config.options.get("max_returns") {
            if let Some(n) = max.as_integer() {
                self.max_returns = n as usize;
            }
        }
        Ok(())
    }
}

fn count_returns(node: Node<'_>) -> usize {
    let mut count = 0;

    if node.kind() == "return_statement" {
        count += 1;
    }

    // Don't recurse into nested function definitions
    if node.kind() != "function_definition" || node.parent().is_none() {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "function_definition" {
                count += count_returns(child);
            }
        }
    } else {
        // For the top-level function, recurse into body
        if let Some(body) = node.child_by_field_name("body") {
            count += count_returns_in_body(body);
        }
    }

    count
}

fn count_returns_in_body(node: Node<'_>) -> usize {
    let mut count = 0;

    if node.kind() == "return_statement" {
        count += 1;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Don't recurse into nested function definitions (lambdas, inner functions)
        if child.kind() != "function_definition" {
            count += count_returns_in_body(child);
        }
    }

    count
}

#[derive(Debug)]
pub struct MaxPublicMethodsRule {
    meta: RuleMetadata,
    max_methods: usize,
}

impl Default for MaxPublicMethodsRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "max-public-methods",
                name: "Maximum Public Methods",
                category: RuleCategory::Design,
                default_severity: Severity::Warning,
                description: "Classes should not have too many public methods",
            },
            max_methods: 20,
        }
    }
}

impl Rule for MaxPublicMethodsRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["class_definition", "source_file"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let mut public_methods = 0;

        let body = if node.kind() == "source_file" {
            Some(node)
        } else {
            node.child_by_field_name("body")
        };

        if let Some(body) = body {
            let mut cursor = body.walk();
            for child in body.children(&mut cursor) {
                if child.kind() == "function_definition" {
                    if let Some(name_node) = child.child_by_field_name("name") {
                        let name = ctx.node_text(name_node);
                        // Public methods don't start with underscore
                        if !name.starts_with('_') {
                            public_methods += 1;
                        }
                    }
                }
            }
        }

        if public_methods > self.max_methods {
            let severity = ctx
                .config()
                .get_rule_severity(self.meta.id, self.meta.default_severity);

            let class_name = if node.kind() == "source_file" {
                "<module>"
            } else {
                node.child_by_field_name("name")
                    .map(|n| ctx.node_text(n))
                    .unwrap_or("<anonymous>")
            };

            ctx.report_node(
                node,
                self.meta.id,
                severity,
                format!(
                    "Class \"{}\" has {} public methods (max {})",
                    class_name, public_methods, self.max_methods
                ),
            );
        }
    }

    fn configure(&mut self, config: &RuleConfig) -> Result<(), String> {
        if let Some(max) = config.options.get("max") {
            if let Some(n) = max.as_integer() {
                self.max_methods = n as usize;
            }
        }
        if let Some(max) = config.options.get("max_methods") {
            if let Some(n) = max.as_integer() {
                self.max_methods = n as usize;
            }
        }
        Ok(())
    }
}
