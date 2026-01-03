use tree_sitter::Node;

use super::expressions::format_expression;
use super::{format_block, format_node};
use crate::format::context::FormatContext;

/// Format class definition.
pub fn format_class_definition(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get class name
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or("_");

    // Get extends clause if present
    let extends = node
        .child_by_field_name("extends")
        .map(|n| format!(" extends {}", ctx.node_text(n)))
        .unwrap_or_default();

    ctx.output
        .push_mapped(format!("{}class {}{}:", indent, name, extends), line);

    // Format body
    if let Some(body) = node.child_by_field_name("body") {
        ctx.indent();
        format_class_body(body, ctx);
        ctx.dedent();
    }
}

/// Format class body (handles member ordering eventually).
fn format_class_body(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    let mut prev_kind: Option<&str> = None;

    for child in children {
        // Add blank line between methods
        if let Some(prev) = prev_kind {
            if needs_blank_line(prev, child.kind()) {
                ctx.output.push_blank_lines(1);
            }
        }

        format_node(child, ctx);
        prev_kind = Some(child.kind());
    }
}

/// Check if we need a blank line between two class members.
fn needs_blank_line(prev: &str, next: &str) -> bool {
    matches!(
        prev,
        "function_definition" | "class_definition" | "enum_definition"
    ) || matches!(
        next,
        "function_definition" | "class_definition" | "enum_definition"
    )
}

/// Format function definition.
pub fn format_function_definition(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Check for static modifier
    let is_static = node
        .children(&mut node.walk())
        .any(|c| c.kind() == "static_keyword");

    // Get function name
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or("_");

    // Get parameters
    let params = node
        .child_by_field_name("parameters")
        .map(|p| format_parameters(p, ctx))
        .unwrap_or_default();

    // Get return type if present
    let return_type = node
        .child_by_field_name("return_type")
        .map(|t| format!(" -> {}", ctx.node_text(t)))
        .unwrap_or_default();

    // Build function signature
    let static_prefix = if is_static { "static " } else { "" };
    ctx.output.push_mapped(
        format!("{}{}func {}({}){}:", indent, static_prefix, name, params, return_type),
        line,
    );

    // Format body
    if let Some(body) = node.child_by_field_name("body") {
        ctx.indent();
        format_function_body(body, ctx);
        ctx.dedent();
    }
}

/// Format function parameters.
fn format_parameters(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let params: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| {
            matches!(
                c.kind(),
                "parameter"
                    | "typed_parameter"
                    | "default_parameter"
                    | "typed_default_parameter"
                    | "identifier"
                    | "typed_identifier"
            )
        })
        .collect();

    let formatted: Vec<String> = params.iter().map(|p| format_parameter(*p, ctx)).collect();
    formatted.join(", ")
}

/// Format a single parameter.
fn format_parameter(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    match node.kind() {
        // Simple identifier parameter (untyped)
        "identifier" => ctx.node_text(node).to_string(),

        // Typed parameter: look for identifier child and type
        "typed_parameter" | "typed_identifier" => {
            // Find the identifier
            let name = node
                .child_by_field_name("name")
                .or_else(|| node.named_child(0).filter(|c| c.kind() == "identifier"))
                .map(|n| ctx.node_text(n))
                .unwrap_or("_");

            // Find the type
            let type_hint = node
                .child_by_field_name("type")
                .map(|t| format!(": {}", ctx.node_text(t).trim()))
                .unwrap_or_default();

            format!("{}{}", name, type_hint)
        }

        // Default parameter (untyped with default value): func foo(x = 5)
        "default_parameter" => {
            let name = node
                .named_child(0)
                .filter(|c| c.kind() == "identifier")
                .map(|n| ctx.node_text(n))
                .unwrap_or("_");

            // Default value is typically the last named child
            let default_val = node
                .named_child(node.named_child_count().saturating_sub(1))
                .filter(|c| c.kind() != "identifier")
                .map(|d| format_expression(d, ctx))
                .unwrap_or_else(|| "null".to_string());

            format!("{} = {}", name, default_val)
        }

        // Typed default parameter: func foo(x: int = 5)
        "typed_default_parameter" => {
            // Structure: identifier, ":", type, "=", value
            let mut cursor = node.walk();
            let children: Vec<_> = node.children(&mut cursor).collect();

            let name = children
                .iter()
                .find(|c| c.kind() == "identifier")
                .map(|n| ctx.node_text(*n))
                .unwrap_or("_");

            let type_hint = children
                .iter()
                .find(|c| c.kind() == "type")
                .map(|t| format!(": {}", ctx.node_text(*t).trim()))
                .unwrap_or_default();

            // Default value is the last named child that isn't identifier or type
            let default_val = children
                .iter()
                .rev()
                .find(|c| c.is_named() && c.kind() != "identifier" && c.kind() != "type")
                .map(|d| format_expression(*d, ctx))
                .unwrap_or_else(|| "null".to_string());

            format!("{}{} = {}", name, type_hint, default_val)
        }

        // Fallback: just use the node text
        _ => ctx.node_text(node).trim().to_string(),
    }
}

/// Format function body.
fn format_function_body(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if children.is_empty() {
        // Empty function body - add pass
        ctx.output.push_line(format!("{}pass", ctx.indent_str()));
        return;
    }

    format_block(node, ctx);
}

/// Format variable statement: `var x = 1` or `var x: int = 1` or `var x := 1`
/// Also handles variables with getter/setter blocks.
pub fn format_variable_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Check for multiline statements (getter/setter, multiline arrays, etc.)
    let is_multiline = node.start_position().row != node.end_position().row;

    // Check for getter/setter block by looking for the setget node in the AST
    let has_setget = node
        .children(&mut node.walk())
        .any(|c| c.kind() == "setget");

    // Check if the value contains comments (which aren't in AST)
    let source = ctx.node_text(node);
    let has_comments = source.contains('#');

    // For getter/setter blocks OR multiline values with comments, preserve verbatim
    // This ensures comments inside arrays/dicts are properly tracked for line mapping
    if is_multiline && (has_setget || has_comments) {
        let start = node.start_position();
        let end = node.end_position();
        for line_idx in start.row..=end.row {
            let line_num = line_idx + 1;
            if let Some(line_content) = ctx.get_source_line(line_num) {
                ctx.output.push_mapped(line_content.to_string(), line_num);
            }
        }
        return;
    }
    // For multiline arrays/dicts without trailing comma, let expression formatter handle it

    // Check for annotations (export, onready)
    // The tree structure is: variable_statement -> annotations -> annotation
    let annotations_prefix = if let Some(annotations_node) = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "annotations")
    {
        let mut cursor = annotations_node.walk();
        let anns: Vec<_> = annotations_node
            .children(&mut cursor)
            .filter(|c| c.kind() == "annotation")
            .map(|a| ctx.node_text(a).trim().to_string())
            .collect();
        if anns.is_empty() {
            String::new()
        } else {
            format!("{} ", anns.join(" "))
        }
    } else {
        String::new()
    };

    // Check if this is an inferred type assignment (:=)
    let source_text = ctx.node_text(node);
    let is_inferred = source_text.contains(":=");

    // Get variable name
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or("_");

    // Get initial value
    let value_node = node.child_by_field_name("value");

    if is_inferred {
        // Inferred type: var x := value
        let value = value_node
            .map(|v| format_expression(v, ctx))
            .unwrap_or_default();
        ctx.output.push_mapped(
            format!("{}{}var {} := {}", indent, annotations_prefix, name, value),
            line,
        );
    } else {
        // Explicit type or no type
        let type_hint = node
            .child_by_field_name("type")
            .map(|t| format!(": {}", ctx.node_text(t).trim()))
            .unwrap_or_default();

        let value = value_node
            .map(|v| format!(" = {}", format_expression(v, ctx)))
            .unwrap_or_default();

        ctx.output.push_mapped(
            format!(
                "{}{}var {}{}{}",
                indent, annotations_prefix, name, type_hint, value
            ),
            line,
        );
    }
}

/// Format const statement: `const X = 1` or `const X: int = 1`
pub fn format_const_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    // Check if the const spans multiple lines (e.g., multiline array)
    // If so, output verbatim to preserve structure and comments
    if node.start_position().row != node.end_position().row {
        let start = node.start_position();
        let end = node.end_position();
        for line_idx in start.row..=end.row {
            let line_num = line_idx + 1;
            if let Some(line_content) = ctx.get_source_line(line_num) {
                ctx.output.push_mapped(line_content.to_string(), line_num);
            }
        }
        return;
    }

    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Check if this is an inferred type constant (:=)
    let source_text = ctx.node_text(node);
    let is_inferred = source_text.contains(":=");

    // Get constant name
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or("_");

    // Get value
    let value_node = node.child_by_field_name("value");

    if is_inferred {
        // Inferred type: const X := value
        let value = value_node
            .map(|v| format_expression(v, ctx))
            .unwrap_or_default();
        ctx.output.push_mapped(
            format!("{}const {} := {}", indent, name, value),
            line,
        );
    } else {
        // Get type hint
        let type_hint = node
            .child_by_field_name("type")
            .map(|t| format!(": {}", ctx.node_text(t)))
            .unwrap_or_default();

        // Get value
        let value = value_node
            .map(|v| format!(" = {}", format_expression(v, ctx)))
            .unwrap_or_default();

        ctx.output
            .push_mapped(format!("{}const {}{}{}", indent, name, type_hint, value), line);
    }
}

/// Format signal statement: `signal my_signal` or `signal my_signal(arg1, arg2)`
pub fn format_signal_statement(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();

    // Get signal name
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or("_");

    // Get parameters if present
    let params = node
        .child_by_field_name("parameters")
        .map(|p| format!("({})", format_signal_parameters(p, ctx)))
        .unwrap_or_default();

    ctx.output
        .push_mapped(format!("{}signal {}{}", indent, name, params), line);
}

/// Format signal parameters.
/// Handles both simple `signal foo(a, b)` and typed `signal foo(a: int, b: String)`.
fn format_signal_parameters(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let params: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| {
            // Include parameter nodes (typed or untyped) and standalone identifiers
            matches!(
                c.kind(),
                "identifier" | "name" | "typed_parameter" | "typed_identifier" | "parameter"
            )
        })
        .collect();

    let formatted: Vec<String> = params.iter().map(|p| format_parameter(*p, ctx)).collect();
    formatted.join(", ")
}

/// Format enum definition.
///
/// Trailing comma determines format:
/// - With trailing comma → multiline (one member per line)
/// - Without trailing comma → single line
pub fn format_enum_definition(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let line = node.start_position().row + 1;
    let indent = ctx.indent_str();
    let source = ctx.node_text(node);

    // Get enum name (optional for anonymous enums)
    let name = node
        .child_by_field_name("name")
        .map(|n| format!(" {}", ctx.node_text(n)))
        .unwrap_or_default();

    // Get enum body
    let body = node.child_by_field_name("body");

    if let Some(body_node) = body {
        let members = collect_enum_members(body_node, ctx);
        if members.is_empty() {
            ctx.output.push_mapped(format!("{}enum{} {{}}", indent, name), line);
        } else {
            // Check if source has trailing comma (before the closing brace)
            let has_trailing_comma = has_trailing_comma_before(source, '}');

            if has_trailing_comma {
                // Multiline format
                let single_indent = ctx.options.indent_style.as_str();
                let inner_indent = format!("{}{}", indent, single_indent);
                ctx.output.push_mapped(format!("{}enum{} {{", indent, name), line);
                for member in &members {
                    // Each member on its own line
                    ctx.output.push_line(format!("{}{},", inner_indent, member));
                }
                ctx.output.push_line(format!("{}}}", indent));
            } else {
                // Single-line format
                ctx.output.push_mapped(
                    format!("{}enum{} {{ {} }}", indent, name, members.join(", ")),
                    line,
                );
            }
        }
    } else {
        ctx.output.push_mapped(format!("{}enum{} {{}}", indent, name), line);
    }
}

/// Check if source has a trailing comma before the specified closing bracket.
fn has_trailing_comma_before(source: &str, close_bracket: char) -> bool {
    let trimmed = source.trim();
    if !trimmed.ends_with(close_bracket) {
        return false;
    }

    // Get content before closing bracket
    let before_close = &trimmed[..trimmed.len() - 1].trim_end();
    before_close.ends_with(',')
}

/// Collect enum member strings.
fn collect_enum_members(node: Node<'_>, ctx: &FormatContext<'_>) -> Vec<String> {
    let mut cursor = node.walk();
    let members: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| c.kind() == "enum_entry" || c.kind() == "enumerator")
        .collect();

    members
        .iter()
        .map(|m| format_enum_member(*m, ctx))
        .collect()
}

/// Format a single enum member.
fn format_enum_member(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let name = node
        .child_by_field_name("name")
        .map(|n| ctx.node_text(n))
        .unwrap_or_else(|| ctx.node_text(node));

    let value = node
        .child_by_field_name("value")
        .map(|v| format!(" = {}", format_expression(v, ctx)));

    if let Some(val) = value {
        format!("{}{}", name, val)
    } else {
        name.to_string()
    }
}

