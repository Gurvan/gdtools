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

/// Count blank lines between two source positions (1-indexed line numbers).
/// Returns the number of consecutive empty lines immediately before next_start_line.
///
/// If there's a comment between statements, we count only the blank lines after
/// the comment (immediately before the next statement), not the total blanks in the gap.
/// This ensures blank lines used to separate a comment from preceding code don't
/// also get counted toward separating from following code.
fn count_source_blank_lines(ctx: &FormatContext<'_>, prev_end_line: usize, next_start_line: usize) -> usize {
    if next_start_line <= prev_end_line + 1 {
        return 0;
    }

    let mut blank_count = 0;
    for line_num in (prev_end_line + 1)..next_start_line {
        if let Some(line) = ctx.get_source_line(line_num) {
            if line.trim().is_empty() {
                blank_count += 1;
            } else {
                // Non-blank line (e.g., a comment) - reset count since we only
                // want consecutive blank lines immediately before the next statement
                blank_count = 0;
            }
        }
    }
    blank_count
}

/// Format the root source_file node.
fn format_source_file(node: Node<'_>, ctx: &mut FormatContext<'_>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    let mut prev_end_line: Option<usize> = None;
    let mut prev_kind: Option<&str> = None;

    for child in children {
        // Calculate blank lines to add
        if let (Some(prev), Some(prev_end)) = (prev_kind, prev_end_line) {
            let child_start_line = child.start_position().row + 1;
            let source_blanks = count_source_blank_lines(ctx, prev_end, child_start_line);
            let required_blanks = blank_lines_between(prev, child.kind(), true);
            // Use the maximum of what was in source vs what's required
            let blank_lines = source_blanks.max(required_blanks).min(2);
            ctx.output.push_blank_lines(blank_lines);
        }

        format_node(child, ctx);
        prev_kind = Some(child.kind());
        prev_end_line = Some(child.end_position().row + 1);
    }
}

/// Determine the minimum blank lines required between two nodes at top level.
///
/// Based on the GDScript style guide:
/// - "Surround functions and class definitions with two blank lines"
/// - One blank line between different declaration sections (signal, enum, const, var)
/// - class_name/extends are grouped together, then one blank line before declarations
fn blank_lines_between(prev: &str, next: &str, is_top_level: bool) -> usize {
    // Comments are handled separately by comment injection - don't add blank lines around them
    // This is especially important for inline comments (on the same line as code)
    if prev == "comment" || next == "comment" {
        return 0;
    }

    // Functions and classes get 2 blank lines at top level, 1 within classes
    let is_function_or_class = |kind: &str| {
        matches!(kind, "function_definition" | "class_definition")
    };

    // If either is a function or class definition, use 2 blank lines at top level
    if is_function_or_class(prev) || is_function_or_class(next) {
        return if is_top_level { 2 } else { 1 };
    }

    // Not top level - no required blank lines between declarations
    if !is_top_level {
        return 0;
    }

    // At top level: determine declaration categories for spacing rules
    // Different categories get 1 blank line between them; same category = no blank line
    let declaration_category = |kind: &str| -> u8 {
        match kind {
            // Category 0: File header (class_name, extends) - grouped together
            "class_name_statement" | "extends_statement" => 0,
            // Category 1: Signals
            "signal_statement" => 1,
            // Category 2: Enums
            "enum_definition" => 2,
            // Category 3: Constants
            "const_statement" => 3,
            // Category 4: Variables (including @export and @onready - we can't distinguish
            // them here since annotations are separate nodes, but grouping all vars together
            // is close enough to the style guide examples)
            "variable_statement" => 4,
            // Other/unknown nodes
            _ => 99,
        }
    };

    let prev_cat = declaration_category(prev);
    let next_cat = declaration_category(next);

    // Annotation followed by something: no blank line (annotation is attached to next decl)
    if prev == "annotation" {
        return 0;
    }

    // Something followed by annotation: add 1 blank line
    // (We can't see what the annotation wraps, so conservatively add spacing)
    if next == "annotation" {
        return 1;
    }

    // Different categories get 1 blank line between them
    if prev_cat != next_cat {
        return 1;
    }

    // Same category: no required blank line
    0
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

    let mut prev_end_line: Option<usize> = None;
    let mut prev_kind: Option<&str> = None;

    for child in children {
        // Add blank lines within blocks (max 1 blank line within blocks)
        if let (Some(prev), Some(prev_end)) = (prev_kind, prev_end_line) {
            let child_start_line = child.start_position().row + 1;
            let source_blanks = count_source_blank_lines(ctx, prev_end, child_start_line);
            let required_blanks = blank_lines_between(prev, child.kind(), false);
            // Within blocks, allow max 1 blank line
            let blank_lines = source_blanks.max(required_blanks).min(1);
            ctx.output.push_blank_lines(blank_lines);
        }

        format_node(child, ctx);
        prev_kind = Some(child.kind());
        prev_end_line = Some(child.end_position().row + 1);
    }
}
