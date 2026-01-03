mod control_flow;
mod declarations;
mod expressions;
mod statements;

use tree_sitter::Node;

use super::context::FormatContext;

/// Format a node and its children.
pub fn format_node(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let start_line = node.start_position().row + 1; // 1-indexed

    // Check if this node is in a skip region
    if ctx.is_skipped(start_line) {
        // Output the original source lines for this node
        format_skipped_node(node, ctx);
        return;
    }

    match node.kind() {
        // Root
        "source" | "source_file" => format_source_file(node, ctx),

        // Declarations
        "class_definition" => declarations::format_class_definition(node, ctx),
        "function_definition" => declarations::format_function_definition(node, ctx),
        "variable_statement" => declarations::format_variable_statement(node, ctx),
        "const_statement" => declarations::format_const_statement(node, ctx),
        "signal_statement" => declarations::format_signal_statement(node, ctx),
        "enum_definition" => declarations::format_enum_definition(node, ctx),

        // Simple statements
        "extends_statement" => statements::format_extends_statement(node, ctx),
        "class_name_statement" => statements::format_class_name_statement(node, ctx),
        "pass_statement" => statements::format_pass_statement(node, ctx),
        "break_statement" => statements::format_break_statement(node, ctx),
        "continue_statement" => statements::format_continue_statement(node, ctx),
        "return_statement" => statements::format_return_statement(node, ctx),
        "expression_statement" => statements::format_expression_statement(node, ctx),

        // Control flow
        "if_statement" => control_flow::format_if_statement(node, ctx),
        "for_statement" => control_flow::format_for_statement(node, ctx),
        "while_statement" => control_flow::format_while_statement(node, ctx),
        "match_statement" => control_flow::format_match_statement(node, ctx),

        // Annotations
        "annotation" => statements::format_annotation(node, ctx),

        // Skip comments (handled separately)
        "comment" => {}

        // For unhandled nodes, just output original text
        _ => {
            format_verbatim(node, ctx);
        }
    }
}

/// Format the root source_file node.
fn format_source_file(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    let mut prev_kind: Option<&str> = None;

    for child in children {
        // Add blank lines between top-level definitions
        if let Some(prev) = prev_kind {
            let blank_lines = blank_lines_between(prev, child.kind(), true);
            ctx.output.push_blank_lines(blank_lines);
        }

        format_node(child, ctx);
        prev_kind = Some(child.kind());
    }
}

/// Determine how many blank lines should separate two nodes.
fn blank_lines_between(prev: &str, next: &str, is_top_level: bool) -> usize {
    // Functions and classes get 2 blank lines at top level, 1 within classes
    let is_definition = |kind: &str| {
        matches!(
            kind,
            "function_definition" | "class_definition" | "enum_definition"
        )
    };

    if is_definition(prev) || is_definition(next) {
        if is_top_level {
            2
        } else {
            1
        }
    } else {
        0
    }
}

/// Output a node verbatim from source (for skipped regions or unhandled nodes).
fn format_verbatim(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let start = node.start_position();
    let end = node.end_position();

    for line_idx in start.row..=end.row {
        let line_num = line_idx + 1; // 1-indexed
        if let Some(line) = ctx.get_source_line(line_num) {
            ctx.output.push_mapped(line.to_string(), line_num);
        }
    }
}

/// Format a node that's in a skip region.
fn format_skipped_node(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    format_verbatim(node, ctx);
}

/// Format a block of statements (function body, if body, etc.).
pub fn format_block(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    let mut prev_kind: Option<&str> = None;

    for child in children {
        // Add blank lines within blocks (but only 1 max)
        if let Some(prev) = prev_kind {
            let blank_lines = blank_lines_between(prev, child.kind(), false);
            ctx.output.push_blank_lines(blank_lines);
        }

        format_node(child, ctx);
        prev_kind = Some(child.kind());
    }
}
