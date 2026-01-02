use std::collections::HashSet;

use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{LintContext, Rule, RuleCategory, RuleMetadata, Severity};

#[derive(Debug)]
pub struct UnnecessaryPassRule {
    meta: RuleMetadata,
}

impl Default for UnnecessaryPassRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "unnecessary-pass",
                name: "Unnecessary Pass",
                category: RuleCategory::Basic,
                default_severity: Severity::Warning,
                description: "pass is unnecessary when block has other statements",
            },
        }
    }
}

impl Rule for UnnecessaryPassRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["pass_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        if let Some(parent) = node.parent() {
            if parent.kind() == "body" || parent.kind() == "block" {
                let sibling_count = parent.named_child_count();
                if sibling_count > 1 {
                    let severity = ctx
                        .config()
                        .get_rule_severity(self.meta.id, self.meta.default_severity);
                    ctx.report_node(
                        node,
                        self.meta.id,
                        severity,
                        "Unnecessary pass statement",
                    );
                }
            }
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct UnusedArgumentRule {
    meta: RuleMetadata,
}

impl Default for UnusedArgumentRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "unused-argument",
                name: "Unused Argument",
                category: RuleCategory::Basic,
                default_severity: Severity::Warning,
                description: "Function arguments should be used",
            },
        }
    }
}

impl Rule for UnusedArgumentRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["function_definition"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let params = collect_parameters(node, ctx);
        if params.is_empty() {
            return;
        }

        let used_names = collect_used_identifiers(node, ctx);

        let severity = ctx
            .config()
            .get_rule_severity(self.meta.id, self.meta.default_severity);

        for (name, name_node) in params {
            if name.starts_with('_') {
                continue;
            }

            if !used_names.contains(&name) {
                ctx.report_node(
                    name_node,
                    self.meta.id,
                    severity,
                    format!("Unused argument \"{}\"", name),
                );
            }
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

fn collect_parameters<'a>(func: Node<'a>, ctx: &LintContext<'_>) -> Vec<(String, Node<'a>)> {
    let mut params = Vec::new();

    if let Some(params_node) = func.child_by_field_name("parameters") {
        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            let name_node = match child.kind() {
                "identifier" => Some(child),
                "typed_parameter" => {
                    // Find the identifier child (first named child)
                    child.named_child(0).filter(|c| c.kind() == "identifier")
                }
                _ => None,
            };

            if let Some(name_node) = name_node {
                let name = ctx.node_text(name_node).to_string();
                params.push((name, name_node));
            }
        }
    }

    params
}

fn collect_used_identifiers(func: Node<'_>, ctx: &LintContext<'_>) -> HashSet<String> {
    let mut used = HashSet::new();

    if let Some(body) = func.child_by_field_name("body") {
        collect_identifiers_recursive(body, ctx, &mut used);
    }

    used
}

fn collect_identifiers_recursive(
    node: Node<'_>,
    ctx: &LintContext<'_>,
    used: &mut HashSet<String>,
) {
    if node.kind() == "identifier" {
        used.insert(ctx.node_text(node).to_string());
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers_recursive(child, ctx, used);
    }
}

#[derive(Debug)]
pub struct ComparisonWithItselfRule {
    meta: RuleMetadata,
}

impl Default for ComparisonWithItselfRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "comparison-with-itself",
                name: "Comparison With Itself",
                category: RuleCategory::Basic,
                default_severity: Severity::Warning,
                description: "Comparing a value with itself is likely a bug",
            },
        }
    }
}

impl Rule for ComparisonWithItselfRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["comparison_operator"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        let child_count = node.named_child_count();
        if child_count < 2 {
            return;
        }

        if let (Some(left), Some(right)) = (node.named_child(0), node.named_child(1)) {
            let left_text = ctx.node_text(left);
            let right_text = ctx.node_text(right);

            if left_text == right_text && !left_text.is_empty() {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    node,
                    self.meta.id,
                    severity,
                    format!("Comparison of \"{}\" with itself", left_text),
                );
            }
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}
