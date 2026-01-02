use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::{Diagnostic, LintContext, Rule, RuleCategory, RuleMetadata, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MemberKind {
    Tool,
    ClassName,
    Extends,
    DocComment,
    Signal,
    Enum,
    Const,
    StaticVar,
    ExportVar,
    Var,
    OnreadyVar,
    VirtualMethod,
    Method,
    InnerClass,
}

impl MemberKind {
    fn from_node(node: Node<'_>, source: &[u8]) -> Option<Self> {
        let node_text = |n: Node<'_>| n.utf8_text(source).unwrap_or("");

        match node.kind() {
            "annotation" => {
                let text = node_text(node);
                if text.starts_with("@tool") {
                    Some(MemberKind::Tool)
                } else if text.starts_with("@export") {
                    Some(MemberKind::ExportVar)
                } else if text.starts_with("@onready") {
                    Some(MemberKind::OnreadyVar)
                } else {
                    None
                }
            }
            "class_name_statement" => Some(MemberKind::ClassName),
            "extends_statement" => Some(MemberKind::Extends),
            "signal_statement" => Some(MemberKind::Signal),
            "enum_definition" => Some(MemberKind::Enum),
            "const_statement" => Some(MemberKind::Const),
            "variable_statement" => {
                if let Some(parent) = node.parent() {
                    if let Some(prev) = find_previous_sibling(parent, node) {
                        if prev.kind() == "annotation" {
                            let text = node_text(prev);
                            if text.starts_with("@export") {
                                return Some(MemberKind::ExportVar);
                            }
                            if text.starts_with("@onready") {
                                return Some(MemberKind::OnreadyVar);
                            }
                            if text.starts_with("@static") {
                                return Some(MemberKind::StaticVar);
                            }
                        }
                    }
                }
                Some(MemberKind::Var)
            }
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let name = node_text(name_node);
                    if is_virtual_method(name) {
                        return Some(MemberKind::VirtualMethod);
                    }
                }
                Some(MemberKind::Method)
            }
            "class_definition" => Some(MemberKind::InnerClass),
            "comment" => {
                let text = node_text(node);
                if text.starts_with("##") {
                    Some(MemberKind::DocComment)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn name(&self) -> &'static str {
        match self {
            MemberKind::Tool => "@tool",
            MemberKind::ClassName => "class_name",
            MemberKind::Extends => "extends",
            MemberKind::DocComment => "doc comment",
            MemberKind::Signal => "signal",
            MemberKind::Enum => "enum",
            MemberKind::Const => "const",
            MemberKind::StaticVar => "static var",
            MemberKind::ExportVar => "@export var",
            MemberKind::Var => "var",
            MemberKind::OnreadyVar => "@onready var",
            MemberKind::VirtualMethod => "virtual method",
            MemberKind::Method => "method",
            MemberKind::InnerClass => "inner class",
        }
    }
}

fn find_previous_sibling<'a>(parent: Node<'a>, target: Node<'a>) -> Option<Node<'a>> {
    let mut cursor = parent.walk();
    let mut prev = None;
    for child in parent.children(&mut cursor) {
        if child.id() == target.id() {
            return prev;
        }
        prev = Some(child);
    }
    None
}

fn is_virtual_method(name: &str) -> bool {
    matches!(
        name,
        "_init"
            | "_ready"
            | "_process"
            | "_physics_process"
            | "_enter_tree"
            | "_exit_tree"
            | "_input"
            | "_unhandled_input"
            | "_notification"
            | "_draw"
            | "_gui_input"
    )
}

#[derive(Debug)]
pub struct ClassDefinitionsOrderRule {
    meta: RuleMetadata,
}

impl Default for ClassDefinitionsOrderRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "class-definitions-order",
                name: "Class Definitions Order",
                category: RuleCategory::Style,
                default_severity: Severity::Warning,
                description:
                    "Class members should follow the recommended order from the style guide",
            },
        }
    }
}

impl Rule for ClassDefinitionsOrderRule {
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
        let source = ctx.source().as_bytes();
        let root = ctx.tree().root_node();

        let diagnostics = self.collect_order_violations(root, source, severity);

        for diagnostic in diagnostics {
            ctx.report(diagnostic);
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

impl ClassDefinitionsOrderRule {
    fn collect_order_violations(
        &self,
        class_node: Node<'_>,
        source: &[u8],
        severity: Severity,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut last_kind: Option<MemberKind> = None;

        let mut cursor = class_node.walk();
        for child in class_node.children(&mut cursor) {
            if let Some(kind) = MemberKind::from_node(child, source) {
                if let Some(prev_kind) = last_kind {
                    if kind < prev_kind {
                        let line = child.start_position().row + 1;
                        let column = child.start_position().column + 1;
                        let diagnostic = Diagnostic::new(
                            self.meta.id,
                            severity,
                            format!("{} should come before {}", kind.name(), prev_kind.name()),
                        )
                        .with_location(line, column);
                        diagnostics.push(diagnostic);
                    }
                }
                last_kind = Some(kind);
            }

            if child.kind() == "class_definition" {
                if let Some(body) = child.child_by_field_name("body") {
                    diagnostics.extend(self.collect_order_violations(body, source, severity));
                }
            }
        }

        diagnostics
    }
}
