use tree_sitter::Node;

use crate::format::context::FormatContext;

/// Format an expression and return it as a string.
pub fn format_expression(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    match node.kind() {
        // Literals
        "integer" | "float" | "string" | "true" | "false" | "null" => {
            ctx.node_text(node).to_string()
        }

        // Identifiers
        "identifier" | "name" => ctx.node_text(node).to_string(),

        // Self reference
        "self" => "self".to_string(),

        // Binary operations
        "binary_operator" => format_binary_operation(node, ctx),

        // Unary operations
        "unary_operator" => format_unary_operation(node, ctx),

        // Comparisons
        "comparison_operator" => format_comparison(node, ctx),

        // Boolean operations
        "boolean_operator" => format_boolean_operation(node, ctx),

        // Function/method calls
        "call" => format_call(node, ctx),

        // Attribute access: obj.attr
        "attribute" => format_attribute(node, ctx),

        // Subscript access: arr[idx]
        "subscript" => format_subscript(node, ctx),

        // Array literal: [1, 2, 3]
        "array" => format_array(node, ctx),

        // Dictionary literal: {a: 1, b: 2}
        "dictionary" => format_dictionary(node, ctx),

        // Parenthesized expression
        "parenthesized_expression" => format_parenthesized(node, ctx),

        // Assignment
        "assignment" => format_assignment(node, ctx),

        // Augmented assignment: +=, -=, etc.
        "augmented_assignment" => format_augmented_assignment(node, ctx),

        // Ternary/conditional expression
        "conditional_expression" | "ternary_expression" => format_ternary(node, ctx),

        // Lambda/anonymous function
        "lambda" => format_lambda(node, ctx),

        // Type cast: x as Type
        "cast" => format_cast(node, ctx),

        // Await expression
        "await" | "await_expression" => format_await(node, ctx),

        // String name (node path): $NodeName
        "get_node" => format_get_node(node, ctx),

        // Default: return source text
        _ => ctx.node_text(node).to_string(),
    }
}


/// Format binary operation: `a + b`, `a * b`, `a not in b`, etc.
fn format_binary_operation(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // Try field names first
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");
    let operator = node.child_by_field_name("operator");

    if let (Some(l), Some(op), Some(r)) = (left, operator, right) {
        let left_text = format_expression(l, ctx);
        let op_text = ctx.node_text(op);
        let right_text = format_expression(r, ctx);
        return format!("{} {} {}", left_text, op_text, right_text);
    }

    // Field names didn't work - look at children directly
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Handle "not in" operator: 4 children (left, "not", "in", right)
    if children.len() == 4
        && children[1].kind() == "not"
        && children[2].kind() == "in"
    {
        let left_text = format_expression(children[0], ctx);
        let right_text = format_expression(children[3], ctx);
        return format!("{} not in {}", left_text, right_text);
    }

    // Handle "is not" operator: 4 children (left, "is", "not", right)
    if children.len() == 4
        && children[1].kind() == "is"
        && children[2].kind() == "not"
    {
        let left_text = format_expression(children[0], ctx);
        let right_text = format_expression(children[3], ctx);
        return format!("{} is not {}", left_text, right_text);
    }

    // Standard binary operations: 3 children (left, operator, right)
    if children.len() >= 3 {
        let left_text = format_expression(children[0], ctx);
        let op_text = ctx.node_text(children[1]).trim();
        let right_text = format_expression(children[2], ctx);
        return format!("{} {} {}", left_text, op_text, right_text);
    }

    // Fallback
    ctx.node_text(node).to_string()
}

/// Format unary operation: `-x`, `not x`, etc.
fn format_unary_operation(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if children.len() >= 2 {
        let op = ctx.node_text(children[0]);
        let operand = format_expression(children[1], ctx);

        // "not" needs a space, "-" and "~" don't
        if op == "not" {
            format!("not {}", operand)
        } else {
            format!("{}{}", op, operand)
        }
    } else {
        ctx.node_text(node).to_string()
    }
}

/// Format comparison: `a == b`, `a < b`, etc.
fn format_comparison(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // Comparisons can be chained: a < b < c
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    let mut parts = Vec::new();
    for (i, child) in children.iter().enumerate() {
        let text = if i % 2 == 0 {
            // Operand
            format_expression(*child, ctx)
        } else {
            // Operator
            ctx.node_text(*child).to_string()
        };
        parts.push(text);
    }

    parts.join(" ")
}

/// Format boolean operation: `a and b`, `a or b`
fn format_boolean_operation(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");
    let operator = node.child_by_field_name("operator");

    match (left, operator, right) {
        (Some(l), Some(op), Some(r)) => {
            let left_text = format_expression(l, ctx);
            let op_text = ctx.node_text(op);
            let right_text = format_expression(r, ctx);
            format!("{} {} {}", left_text, op_text, right_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format function/method call: `func(a, b)` or `obj.method(a, b)`
fn format_call(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // Try field names first
    let function = node.child_by_field_name("function");
    let arguments = node.child_by_field_name("arguments");

    if let (Some(func), Some(args)) = (function, arguments) {
        let func_text = format_expression(func, ctx);
        let args_text = format_arguments(args, ctx);
        return format!("{}({})", func_text, args_text);
    }

    if let Some(func) = function {
        let func_text = format_expression(func, ctx);
        return format!("{}()", func_text);
    }

    // Field names didn't work - try looking at children directly
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Find the function part (first child that's not punctuation)
    let func_node = children
        .iter()
        .find(|c| !matches!(c.kind(), "(" | ")" | ","));

    // Find the arguments (look for argument_list or just collect args)
    let args_node = children.iter().find(|c| c.kind() == "argument_list" || c.kind() == "arguments");

    if let Some(func) = func_node {
        let func_text = format_expression(*func, ctx);
        if let Some(args) = args_node {
            let args_text = format_arguments(*args, ctx);
            return format!("{}({})", func_text, args_text);
        }
        // Collect arguments directly from children
        let args: Vec<_> = children
            .iter()
            .filter(|c| !matches!(c.kind(), "(" | ")" | "," | "identifier" | "attribute"))
            .filter(|c| c.start_byte() != func.start_byte())
            .map(|c| format_expression(*c, ctx))
            .collect();
        if args.is_empty() {
            return format!("{}()", func_text);
        }
        return format!("{}({})", func_text, args.join(", "));
    }

    // Fallback
    ctx.node_text(node).to_string()
}

/// Format argument list.
fn format_arguments(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let children: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| c.kind() != "(" && c.kind() != ")" && c.kind() != ",")
        .collect();

    let args: Vec<String> = children.iter().map(|c| format_expression(*c, ctx)).collect();
    args.join(", ")
}

/// Format attribute access: `obj.attr`
fn format_attribute(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let object = node.child_by_field_name("object");
    let attribute = node.child_by_field_name("attribute");

    match (object, attribute) {
        (Some(obj), Some(attr)) => {
            let obj_text = format_expression(obj, ctx);
            let attr_text = ctx.node_text(attr);
            format!("{}.{}", obj_text, attr_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format subscript access: `arr[idx]`
fn format_subscript(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let value = node.child_by_field_name("value");
    let subscript = node.child_by_field_name("subscript");

    match (value, subscript) {
        (Some(val), Some(sub)) => {
            let val_text = format_expression(val, ctx);
            let sub_text = format_expression(sub, ctx);
            format!("{}[{}]", val_text, sub_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format array literal: `[1, 2, 3]`
///
/// Preserves multiline arrays to maintain readability and inline comments.
fn format_array(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let children: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| c.kind() != "[" && c.kind() != "]" && c.kind() != ",")
        .collect();

    if children.is_empty() {
        return "[]".to_string();
    }

    // Check if the source array was multiline
    let is_multiline = node.start_position().row != node.end_position().row;

    if is_multiline {
        // Preserve multiline arrays verbatim to keep structure and comments
        return ctx.node_text(node).to_string();
    }

    let elements: Vec<String> = children.iter().map(|c| format_expression(*c, ctx)).collect();
    format!("[{}]", elements.join(", "))
}

/// Format dictionary literal: `{ a: 1, b: 2 }`
///
/// Per the GDScript style guide:
/// - Single-line dictionaries should have space after `{` and before `}`
/// - Multi-line dictionaries have each entry on its own line with trailing comma
fn format_dictionary(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    // Dictionary entries can be "pair" nodes - collect all non-punctuation children
    let children: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| !matches!(c.kind(), "{" | "}" | ","))
        .collect();

    if children.is_empty() {
        return "{}".to_string();
    }

    // Check if the source dictionary was multiline (spans multiple lines)
    let is_multiline = node.start_position().row != node.end_position().row;

    if is_multiline {
        // Format as multiline dictionary with each entry on its own line
        let indent = ctx.indent_str();
        let single_indent = ctx.options.indent_style.as_str();
        let inner_indent = format!("{}{}", indent, single_indent);
        let pairs: Vec<String> = children.iter().map(|c| format_pair(*c, ctx)).collect();
        format!(
            "{{\n{}{},\n{}}}",
            inner_indent,
            pairs.join(&format!(",\n{}", inner_indent)),
            indent
        )
    } else {
        // Single-line: add space after { and before } for readability
        let pairs: Vec<String> = children.iter().map(|c| format_pair(*c, ctx)).collect();
        format!("{{ {} }}", pairs.join(", "))
    }
}

/// Format a key-value pair in a dictionary.
fn format_pair(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // Try field names first
    let key = node.child_by_field_name("key");
    let value = node.child_by_field_name("value");

    if let (Some(k), Some(v)) = (key, value) {
        let key_text = format_expression(k, ctx);
        let val_text = format_expression(v, ctx);
        return format!("{}: {}", key_text, val_text);
    }

    // Fallback: look at children directly
    // Pair structure is typically: key, ":", value
    let mut cursor = node.walk();
    let children: Vec<_> = node
        .children(&mut cursor)
        .filter(|c| c.kind() != ":")
        .collect();

    if children.len() >= 2 {
        let key_text = format_expression(children[0], ctx);
        let val_text = format_expression(children[1], ctx);
        return format!("{}: {}", key_text, val_text);
    }

    // Last resort: return original text
    ctx.node_text(node).to_string()
}

/// Format parenthesized expression: `(expr)`
fn format_parenthesized(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let inner = node
        .children(&mut cursor)
        .find(|c| c.kind() != "(" && c.kind() != ")");

    if let Some(expr) = inner {
        format!("({})", format_expression(expr, ctx))
    } else {
        ctx.node_text(node).to_string()
    }
}

/// Format assignment: `x = y`
fn format_assignment(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");

    match (left, right) {
        (Some(l), Some(r)) => {
            let left_text = format_expression(l, ctx);
            let right_text = format_expression(r, ctx);
            format!("{} = {}", left_text, right_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format augmented assignment: `x += y`
fn format_augmented_assignment(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let left = node.child_by_field_name("left");
    let right = node.child_by_field_name("right");
    let operator = node.child_by_field_name("operator");

    match (left, operator, right) {
        (Some(l), Some(op), Some(r)) => {
            let left_text = format_expression(l, ctx);
            let op_text = ctx.node_text(op);
            let right_text = format_expression(r, ctx);
            format!("{} {} {}", left_text, op_text, right_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format ternary/conditional expression: `x if cond else y`
fn format_ternary(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // GDScript ternary: value_if_true if condition else value_if_false
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    // Try to extract parts
    let true_val = node.child_by_field_name("true");
    let condition = node.child_by_field_name("condition");
    let false_val = node.child_by_field_name("false");

    match (true_val, condition, false_val) {
        (Some(t), Some(c), Some(f)) => {
            let true_text = format_expression(t, ctx);
            let cond_text = format_expression(c, ctx);
            let false_text = format_expression(f, ctx);
            format!("{} if {} else {}", true_text, cond_text, false_text)
        }
        _ => {
            // Fallback: reconstruct from children
            if children.len() >= 5 {
                let true_text = format_expression(children[0], ctx);
                let cond_text = format_expression(children[2], ctx);
                let false_text = format_expression(children[4], ctx);
                format!("{} if {} else {}", true_text, cond_text, false_text)
            } else {
                ctx.node_text(node).to_string()
            }
        }
    }
}

/// Format lambda: `func(x): return x * 2`
fn format_lambda(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    // For now, just return source text (lambdas are complex)
    ctx.node_text(node).to_string()
}

/// Format type cast: `x as Type`
fn format_cast(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let value = node.child_by_field_name("value");
    let cast_type = node.child_by_field_name("type");

    match (value, cast_type) {
        (Some(v), Some(t)) => {
            let val_text = format_expression(v, ctx);
            let type_text = ctx.node_text(t);
            format!("{} as {}", val_text, type_text)
        }
        _ => ctx.node_text(node).to_string(),
    }
}

/// Format await expression: `await signal`
fn format_await(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    let mut cursor = node.walk();
    let expr = node.children(&mut cursor).find(|c| c.kind() != "await");

    if let Some(e) = expr {
        format!("await {}", format_expression(e, ctx))
    } else {
        ctx.node_text(node).to_string()
    }
}

/// Format get_node: `$NodePath` or `%UniqueNode`
fn format_get_node(node: Node<'_>, ctx: &FormatContext<'_>) -> String {
    ctx.node_text(node).to_string()
}
