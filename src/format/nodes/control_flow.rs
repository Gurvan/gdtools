use tree_sitter::Node;

use super::expressions::format_expression;
use super::format_block;
use crate::format::context::FormatContext;

/// Format if statement.
pub fn format_if_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    // Check if this is a single-line if statement (body on same line as condition)
    // If so, output verbatim to preserve the structure
    if node.start_position().row == node.end_position().row {
        let start = node.start_position();
        let line_num = start.row + 1;
        if let Some(line_content) = ctx.get_source_line(line_num) {
            ctx.output.push_mapped(line_content.to_string(), line_num);
        }
        return;
    }

    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get condition
    let condition = node
        .child_by_field_name("condition")
        .map(|c| format_expression(c, ctx))
        .unwrap_or_else(|| "true".to_string());

    ctx.output
        .push_mapped(format!("{}if {}:", indent, condition), line);

    // Format consequence (then block) - try multiple field names
    let body = node
        .child_by_field_name("consequence")
        .or_else(|| node.child_by_field_name("body"));

    if let Some(body_node) = body {
        ctx.indent();
        format_block(body_node, ctx);
        ctx.dedent();
    } else {
        // Try to find body by looking at children directly
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        // Find statements that are part of the if body (after the condition, before elif/else)
        ctx.indent();
        for child in &children {
            let kind = child.kind();
            if kind != "if" && kind != "elif_clause" && kind != "else_clause"
                && !is_condition_node(kind) && kind != ":"
            {
                super::format_node(*child, ctx);
            }
        }
        ctx.dedent();
    }

    // Handle elif/else branches
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "elif_clause" => format_elif_clause(child, ctx),
            "else_clause" => format_else_clause(child, ctx),
            _ => {}
        }
    }
}

fn is_condition_node(kind: &str) -> bool {
    matches!(kind, "binary_operator" | "comparison_operator" | "boolean_operator"
        | "identifier" | "true" | "false" | "call" | "parenthesized_expression")
}

/// Format elif clause.
fn format_elif_clause(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    let condition = node
        .child_by_field_name("condition")
        .map(|c| format_expression(c, ctx))
        .unwrap_or_else(|| "true".to_string());

    ctx.output
        .push_mapped(format!("{}elif {}:", indent, condition), line);

    // Try to find body via field name or fallback
    let body = node
        .child_by_field_name("consequence")
        .or_else(|| node.child_by_field_name("body"));

    if let Some(body_node) = body {
        ctx.indent();
        format_block(body_node, ctx);
        ctx.dedent();
    } else {
        // Fallback: find body statements directly in children
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        ctx.indent();
        for child in &children {
            let kind = child.kind();
            if kind != "elif" && !is_condition_node(kind) && kind != ":" {
                super::format_node(*child, ctx);
            }
        }
        ctx.dedent();
    }
}

/// Format else clause.
fn format_else_clause(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    ctx.output.push_mapped(format!("{}else:", indent), line);

    if let Some(body) = node.child_by_field_name("body") {
        ctx.indent();
        format_block(body, ctx);
        ctx.dedent();
    } else {
        // Try to find the body as a direct child
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() != "else" {
                ctx.indent();
                format_block(child, ctx);
                ctx.dedent();
                break;
            }
        }
    }
}

/// Format for statement.
pub fn format_for_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get loop variable
    let var = node
        .child_by_field_name("variable")
        .or_else(|| node.child_by_field_name("left"))
        .map(|v| node_text(v, ctx))
        .unwrap_or("_");

    // Get iterable
    let iterable = node
        .child_by_field_name("value")
        .or_else(|| node.child_by_field_name("right"))
        .map(|i| format_expression(i, ctx))
        .unwrap_or_else(|| "[]".to_string());

    ctx.output
        .push_mapped(format!("{}for {} in {}:", indent, var, iterable), line);

    // Format body
    if let Some(body) = node.child_by_field_name("body") {
        ctx.indent();
        format_block(body, ctx);
        ctx.dedent();
    }
}

/// Format while statement.
pub fn format_while_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get condition
    let condition = node
        .child_by_field_name("condition")
        .map(|c| format_expression(c, ctx))
        .unwrap_or_else(|| "true".to_string());

    ctx.output
        .push_mapped(format!("{}while {}:", indent, condition), line);

    // Format body
    if let Some(body) = node.child_by_field_name("body") {
        ctx.indent();
        format_block(body, ctx);
        ctx.dedent();
    }
}

/// Format match statement.
///
/// Match statements are complex and can have:
/// - Multiple patterns per branch: `"a", "b":`
/// - Single-line bodies: `0: foo()`
/// - Complex pattern syntax
///
/// For now, output match statements verbatim to preserve all cases correctly.
pub fn format_match_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    // Output verbatim to preserve all patterns and body formatting
    let start = node.start_position();
    let end = node.end_position();
    for line_idx in start.row..=end.row {
        let line_num = line_idx + 1;
        if let Some(line_content) = ctx.get_source_line(line_num) {
            ctx.output.push_mapped(line_content.to_string(), line_num);
        }
    }
}

/// Format a match branch.
#[allow(dead_code)]
fn format_match_branch(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get pattern - try field name first, then first named child
    let pattern = node
        .child_by_field_name("pattern")
        .or_else(|| node.named_child(0))
        .map(|p| format_pattern(p, ctx))
        .unwrap_or_else(|| "_".to_string());

    ctx.output
        .push_mapped(format!("{}{}:", indent, pattern), line);

    // Format body - try field name first, then look for body node
    let body = node
        .child_by_field_name("body")
        .or_else(|| {
            node.children(&mut node.walk())
                .find(|c| c.kind() == "body")
        });

    if let Some(body_node) = body {
        ctx.indent();
        format_block(body_node, ctx);
        ctx.dedent();
    }
}

/// Format a match pattern.
#[allow(dead_code)]
fn format_pattern(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    match node.kind() {
        "pattern_binding" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| node_text(n, ctx))
                .unwrap_or("_");
            format!("var {}", name)
        }
        "pattern_array" => {
            let mut cursor = node.walk();
            let elements: Vec<_> = node
                .children(&mut cursor)
                .filter(|c| c.kind() != "[" && c.kind() != "]" && c.kind() != ",")
                .collect();
            let patterns: Vec<String> = elements.iter().map(|e| format_pattern(*e, ctx)).collect();
            format!("[{}]", patterns.join(", "))
        }
        "pattern_dictionary" => {
            // Dictionary pattern matching
            node_text(node, ctx).to_string()
        }
        _ => {
            // Literal or identifier pattern
            format_expression(node, ctx)
        }
    }
}

fn node_text<'a>(node: Node<'_>, ctx: &'a FormatContext<'_>) -> &'a str {
    let start = node.start_byte();
    let end = node.end_byte();
    &ctx.source[start..end]
}
