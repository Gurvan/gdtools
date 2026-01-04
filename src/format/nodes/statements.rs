use tree_sitter::Node;

use super::expressions::format_expression;
use crate::format::context::FormatContext;

/// Format extends statement: `extends Node2D`
pub fn format_extends_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get the type being extended - try different field names
    let type_node = node
        .child_by_field_name("value")
        .or_else(|| node.child_by_field_name("type"));

    // If field names don't work, find the first non-keyword child
    let type_node = type_node.or_else(|| {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        children.into_iter().find(|c| c.kind() != "extends")
    });

    if let Some(tn) = type_node {
        let type_text = ctx.node_text(tn).trim();
        ctx.output
            .push_mapped(format!("{}extends {}", indent, type_text), line);
    } else {
        // Fallback: use source text
        let text = ctx.node_text(node);
        ctx.output
            .push_mapped(format!("{}{}", indent, text.trim()), line);
    }
}

/// Format class_name statement: `class_name MyClass`
pub fn format_class_name_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Try field name first, then look for identifier child
    let name_node = node.child_by_field_name("name").or_else(|| {
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();
        children
            .into_iter()
            .find(|c| c.kind() == "identifier" || c.kind() == "name")
    });

    if let Some(nn) = name_node {
        let name = ctx.node_text(nn).trim();
        ctx.output
            .push_mapped(format!("{}class_name {}", indent, name), line);
    } else {
        let text = ctx.node_text(node);
        ctx.output
            .push_mapped(format!("{}{}", indent, text.trim()), line);
    }
}

/// Format pass statement.
pub fn format_pass_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    ctx.output
        .push_mapped(format!("{}pass", ctx.indent_str()), line);
}

/// Format break statement.
pub fn format_break_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    ctx.output
        .push_mapped(format!("{}break", ctx.indent_str()), line);
}

/// Format continue statement.
pub fn format_continue_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    ctx.output
        .push_mapped(format!("{}continue", ctx.indent_str()), line);
}

/// Format return statement: `return` or `return value`
pub fn format_return_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Check for return value
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Find the expression (skip "return" keyword)
    let expr = children.iter().find(|c| c.kind() != "return");

    if let Some(expr_node) = expr {
        let expr_text = format_expression(*expr_node, ctx);
        ctx.output
            .push_mapped(format!("{}return {}", indent, expr_text), line);
    } else {
        ctx.output.push_mapped(format!("{}return", indent), line);
    }
}

/// Format expression statement (standalone expression).
pub fn format_expression_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    if let Some(expr) = node.child(0) {
        let expr_text = format_expression(expr, ctx);
        ctx.output
            .push_mapped(format!("{}{}", indent, expr_text), line);
    }
}

/// Format annotation: `@export`, `@onready`, etc.
pub fn format_annotation(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get the annotation text
    let text = ctx.node_text(node).trim().to_string();
    ctx.output.push_mapped(format!("{}{}", indent, text), line);
}
