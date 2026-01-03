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

/// Extract the annotation name from an annotation node.
/// Annotation nodes have an identifier child containing the name (e.g., "tool", "export").
fn get_annotation_name<'a>(node: Node<'a>, source: &'a [u8]) -> Option<&'a str> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return child.utf8_text(source).ok();
        }
    }
    None
}

impl MemberKind {
    fn from_node(node: Node<'_>, source: &[u8]) -> Option<Self> {
        let node_text = |n: Node<'_>| n.utf8_text(source).unwrap_or("");

        match node.kind() {
            "annotation" => {
                match get_annotation_name(node, source) {
                    Some("tool") => Some(MemberKind::Tool),
                    Some("export" | "export_category" | "export_group" | "export_subgroup"
                         | "export_color_no_alpha" | "export_dir" | "export_enum"
                         | "export_exp_easing" | "export_file" | "export_flags"
                         | "export_flags_2d_navigation" | "export_flags_2d_physics"
                         | "export_flags_2d_render" | "export_flags_3d_navigation"
                         | "export_flags_3d_physics" | "export_flags_3d_render"
                         | "export_global_dir" | "export_global_file" | "export_multiline"
                         | "export_node_path" | "export_placeholder" | "export_range"
                         | "export_storage" | "export_custom") => Some(MemberKind::ExportVar),
                    Some("onready") => Some(MemberKind::OnreadyVar),
                    _ => None,
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
                            match get_annotation_name(prev, source) {
                                Some("export" | "export_category" | "export_group" | "export_subgroup"
                                     | "export_color_no_alpha" | "export_dir" | "export_enum"
                                     | "export_exp_easing" | "export_file" | "export_flags"
                                     | "export_flags_2d_navigation" | "export_flags_2d_physics"
                                     | "export_flags_2d_render" | "export_flags_3d_navigation"
                                     | "export_flags_3d_physics" | "export_flags_3d_render"
                                     | "export_global_dir" | "export_global_file" | "export_multiline"
                                     | "export_node_path" | "export_placeholder" | "export_range"
                                     | "export_storage" | "export_custom") => {
                                    return Some(MemberKind::ExportVar);
                                }
                                Some("onready") => return Some(MemberKind::OnreadyVar),
                                Some("static") => return Some(MemberKind::StaticVar),
                                _ => {}
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

#[derive(Debug)]
pub struct NoElifReturnRule {
    meta: RuleMetadata,
}

impl Default for NoElifReturnRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "no-elif-return",
                name: "No Elif After Return",
                category: RuleCategory::Style,
                default_severity: Severity::Warning,
                description: "Use else instead of elif when the if branch returns",
            },
        }
    }
}

impl Rule for NoElifReturnRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["if_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Check if the if branch ends with a return
        // The if body is in "body" field (first body child of if_statement)
        if let Some(body) = node.child_by_field_name("body") {
            if !block_ends_with_return(body) {
                return;
            }

            // Check for elif branches
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "elif_clause" {
                    let severity = ctx
                        .config()
                        .get_rule_severity(self.meta.id, self.meta.default_severity);
                    ctx.report_node(
                        child,
                        self.meta.id,
                        severity,
                        "Use 'if' instead of 'elif' when the previous branch returns",
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
pub struct NoElseReturnRule {
    meta: RuleMetadata,
}

impl Default for NoElseReturnRule {
    fn default() -> Self {
        Self {
            meta: RuleMetadata {
                id: "no-else-return",
                name: "No Else After Return",
                category: RuleCategory::Style,
                default_severity: Severity::Warning,
                description: "Unnecessary else after return statement",
            },
        }
    }
}

impl Rule for NoElseReturnRule {
    fn meta(&self) -> &RuleMetadata {
        &self.meta
    }

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        Some(&["if_statement"])
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>) {
        // Check if the if branch (and all elif branches) end with a return
        if let Some(body) = node.child_by_field_name("body") {
            if !block_ends_with_return(body) {
                return;
            }
        } else {
            return;
        }

        // Check all elif branches
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "elif_clause" {
                if let Some(body) = child.child_by_field_name("body") {
                    if !block_ends_with_return(body) {
                        return;
                    }
                }
            }
        }

        // If we get here, all if/elif branches return
        // Check for else clause
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "else_clause" {
                let severity = ctx
                    .config()
                    .get_rule_severity(self.meta.id, self.meta.default_severity);
                ctx.report_node(
                    child,
                    self.meta.id,
                    severity,
                    "Unnecessary 'else' after 'return'",
                );
            }
        }
    }

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}

fn block_ends_with_return(block: Node<'_>) -> bool {
    let mut cursor = block.walk();
    let children: Vec<_> = block.children(&mut cursor).collect();

    if let Some(last) = children.last() {
        if last.kind() == "return_statement" {
            return true;
        }
        // Check if it's an if statement where all branches return
        if last.kind() == "if_statement" {
            return all_branches_return(*last);
        }
    }
    false
}

fn all_branches_return(if_node: Node<'_>) -> bool {
    // Check if branch
    if let Some(body) = if_node.child_by_field_name("body") {
        if !block_ends_with_return(body) {
            return false;
        }
    } else {
        return false;
    }

    let mut has_else = false;
    let mut cursor = if_node.walk();
    for child in if_node.children(&mut cursor) {
        match child.kind() {
            "elif_clause" => {
                if let Some(body) = child.child_by_field_name("body") {
                    if !block_ends_with_return(body) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            "else_clause" => {
                has_else = true;
                if let Some(body) = child.child_by_field_name("body") {
                    if !block_ends_with_return(body) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            _ => {}
        }
    }

    // Must have an else clause for all branches to return
    has_else
}
