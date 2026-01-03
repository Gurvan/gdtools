//! AST equivalence checking for formatter verification.
//!
//! This module provides functions to compare two ASTs and verify they are
//! structurally equivalent, ignoring whitespace and position information.
//! Used in tests to ensure the formatter doesn't change program semantics.

use tree_sitter::{Node, Tree};

/// Result of comparing two ASTs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstCheckResult {
    /// The ASTs are structurally equivalent.
    Equivalent,
    /// The ASTs differ at the specified path.
    Different {
        /// Path to the differing node (e.g., "function_definition[0].body.if_statement")
        path: String,
        /// Description of the difference
        difference: String,
    },
}

impl AstCheckResult {
    /// Returns true if the ASTs are equivalent.
    pub fn is_equivalent(&self) -> bool {
        matches!(self, AstCheckResult::Equivalent)
    }
}

/// Compare two ASTs for structural equivalence.
///
/// This compares:
/// - Node kinds
/// - Named child counts
/// - Field names and their values
/// - Literal text for terminal nodes
///
/// This ignores:
/// - Byte offsets
/// - Row/column positions
/// - Whitespace
pub fn compare_ast(original: &Tree, formatted: &Tree) -> AstCheckResult {
    compare_nodes(
        original.root_node(),
        formatted.root_node(),
        original.root_node(),
        formatted.root_node(),
        String::new(),
    )
}

/// Compare two nodes recursively.
fn compare_nodes<'a>(
    orig: Node<'a>,
    fmt: Node<'a>,
    orig_root: Node<'a>,
    fmt_root: Node<'a>,
    path: String,
) -> AstCheckResult {
    // Compare node kinds
    if orig.kind() != fmt.kind() {
        return AstCheckResult::Different {
            path,
            difference: format!(
                "node kind differs: '{}' vs '{}'",
                orig.kind(),
                fmt.kind()
            ),
        };
    }

    // For terminal nodes (no named children), compare the text content
    // But only for nodes that represent actual values (literals, identifiers)
    if orig.named_child_count() == 0 && fmt.named_child_count() == 0 {
        // These are leaf nodes - their meaning comes from the source text
        // We need to compare the actual text for literals and identifiers
        if is_value_node(orig.kind()) {
            let orig_text = node_text(orig, orig_root);
            let fmt_text = node_text(fmt, fmt_root);
            if orig_text != fmt_text {
                return AstCheckResult::Different {
                    path,
                    difference: format!(
                        "{} value differs: '{}' vs '{}'",
                        orig.kind(),
                        orig_text,
                        fmt_text
                    ),
                };
            }
        }
    }

    // Compare named child count
    if orig.named_child_count() != fmt.named_child_count() {
        return AstCheckResult::Different {
            path,
            difference: format!(
                "named child count differs: {} vs {}",
                orig.named_child_count(),
                fmt.named_child_count()
            ),
        };
    }

    // Compare named children recursively
    let mut orig_cursor = orig.walk();
    let mut fmt_cursor = fmt.walk();

    let orig_children: Vec<_> = orig.named_children(&mut orig_cursor).collect();
    let fmt_children: Vec<_> = fmt.named_children(&mut fmt_cursor).collect();

    for (i, (orig_child, fmt_child)) in orig_children.iter().zip(fmt_children.iter()).enumerate() {
        let child_path = if path.is_empty() {
            format!("{}[{}]", orig_child.kind(), i)
        } else {
            format!("{}.{}[{}]", path, orig_child.kind(), i)
        };

        let result = compare_nodes(*orig_child, *fmt_child, orig_root, fmt_root, child_path);
        if !result.is_equivalent() {
            return result;
        }
    }

    AstCheckResult::Equivalent
}

/// Check if a node kind represents a value that should be compared textually.
fn is_value_node(kind: &str) -> bool {
    matches!(
        kind,
        "identifier"
            | "name"
            | "integer"
            | "float"
            | "string"
            | "true"
            | "false"
            | "null"
            | "self"
            | "type"
    )
}

/// Extract the source text for a node.
///
/// Note: This requires access to the original source, which we get via the root node.
/// In practice, we'll need to pass the source string separately.
fn node_text<'a>(_node: Node<'a>, _root: Node<'a>) -> &'a str {
    // This is a placeholder - we need the actual source text
    // The real implementation will need the source string
    ""
}

/// Compare two ASTs with access to their source strings.
pub fn compare_ast_with_source(
    original_tree: &Tree,
    original_source: &str,
    formatted_tree: &Tree,
    formatted_source: &str,
) -> AstCheckResult {
    compare_nodes_with_source(
        original_tree.root_node(),
        original_source,
        formatted_tree.root_node(),
        formatted_source,
        String::new(),
    )
}

/// Compare two nodes recursively with source access.
fn compare_nodes_with_source(
    orig: Node<'_>,
    orig_source: &str,
    fmt: Node<'_>,
    fmt_source: &str,
    path: String,
) -> AstCheckResult {
    // Compare node kinds
    if orig.kind() != fmt.kind() {
        return AstCheckResult::Different {
            path,
            difference: format!(
                "node kind differs: '{}' vs '{}'",
                orig.kind(),
                fmt.kind()
            ),
        };
    }

    // For terminal nodes, compare text content
    if orig.named_child_count() == 0 && fmt.named_child_count() == 0 {
        if is_value_node(orig.kind()) {
            let orig_text = &orig_source[orig.start_byte()..orig.end_byte()];
            let fmt_text = &fmt_source[fmt.start_byte()..fmt.end_byte()];
            if orig_text != fmt_text {
                return AstCheckResult::Different {
                    path,
                    difference: format!(
                        "{} value differs: '{}' vs '{}'",
                        orig.kind(),
                        orig_text,
                        fmt_text
                    ),
                };
            }
        }
    }

    // Compare named child count
    if orig.named_child_count() != fmt.named_child_count() {
        return AstCheckResult::Different {
            path,
            difference: format!(
                "named child count differs: {} vs {}",
                orig.named_child_count(),
                fmt.named_child_count()
            ),
        };
    }

    // Compare named children recursively
    let mut orig_cursor = orig.walk();
    let mut fmt_cursor = fmt.walk();

    let orig_children: Vec<_> = orig.named_children(&mut orig_cursor).collect();
    let fmt_children: Vec<_> = fmt.named_children(&mut fmt_cursor).collect();

    for (i, (orig_child, fmt_child)) in orig_children.iter().zip(fmt_children.iter()).enumerate() {
        let child_path = if path.is_empty() {
            format!("{}[{}]", orig_child.kind(), i)
        } else {
            format!("{}.{}[{}]", path, orig_child.kind(), i)
        };

        let result =
            compare_nodes_with_source(*orig_child, orig_source, *fmt_child, fmt_source, child_path);
        if !result.is_equivalent() {
            return result;
        }
    }

    AstCheckResult::Equivalent
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_gdscript::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_identical_code() {
        let source = "var x = 1\n";
        let tree1 = parse(source);
        let tree2 = parse(source);
        assert_eq!(
            compare_ast_with_source(&tree1, source, &tree2, source),
            AstCheckResult::Equivalent
        );
    }

    #[test]
    fn test_whitespace_difference() {
        let source1 = "var x=1\n";
        let source2 = "var x = 1\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        assert_eq!(
            compare_ast_with_source(&tree1, source1, &tree2, source2),
            AstCheckResult::Equivalent
        );
    }

    #[test]
    fn test_indentation_difference() {
        let source1 = "func foo():\n  pass\n";
        let source2 = "func foo():\n\tpass\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        assert_eq!(
            compare_ast_with_source(&tree1, source1, &tree2, source2),
            AstCheckResult::Equivalent
        );
    }

    #[test]
    fn test_different_values() {
        let source1 = "var x = 1\n";
        let source2 = "var x = 2\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        let result = compare_ast_with_source(&tree1, source1, &tree2, source2);
        assert!(!result.is_equivalent());
    }

    #[test]
    fn test_different_identifiers() {
        let source1 = "var x = 1\n";
        let source2 = "var y = 1\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        let result = compare_ast_with_source(&tree1, source1, &tree2, source2);
        assert!(!result.is_equivalent());
    }

    #[test]
    fn test_different_structure() {
        let source1 = "var x = 1\n";
        let source2 = "var x: int = 1\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        let result = compare_ast_with_source(&tree1, source1, &tree2, source2);
        assert!(!result.is_equivalent());
    }

    #[test]
    fn test_multiline_vs_singleline_dict() {
        let source1 = "{a: 1, b: 2}\n";
        let source2 = "{\n\ta: 1,\n\tb: 2,\n}\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        // Dictionary structure should be the same regardless of formatting
        assert_eq!(
            compare_ast_with_source(&tree1, source1, &tree2, source2),
            AstCheckResult::Equivalent
        );
    }

    #[test]
    fn test_function_with_different_spacing() {
        let source1 = "func foo(a:int,b:String)->void:\n\tpass\n";
        let source2 = "func foo(a: int, b: String) -> void:\n\tpass\n";
        let tree1 = parse(source1);
        let tree2 = parse(source2);
        assert_eq!(
            compare_ast_with_source(&tree1, source1, &tree2, source2),
            AstCheckResult::Equivalent
        );
    }
}
