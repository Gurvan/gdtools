//! Code reordering according to the GDScript style guide.
//!
//! This module reorders class members to follow the official ordering:
//! 1. @tool, @icon, @static_unload
//! 2. class_name
//! 3. extends
//! 4. ## doc comment
//! 5. signals
//! 6. enums
//! 7. constants
//! 8. static variables
//! 9. @export variables
//! 10. remaining regular variables
//! 11. @onready variables
//! 12. _static_init()
//! 13. remaining static methods
//! 14. virtual methods (_init, _enter_tree, _ready, _process, _physics_process, others)
//! 15. overridden custom methods
//! 16. remaining methods
//! 17. subclasses

use tree_sitter::Node;

use crate::parser;

use super::skip_regions::SkipRegions;
use super::FormatError;

/// Classification of class members for reordering.
/// The order of variants determines sort priority (lower = earlier in file).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberKind {
    // 01. File-level annotations
    Tool,
    Icon,
    StaticUnload,

    // 02. class_name
    ClassName,

    // 03. extends
    Extends,

    // 04. Doc comment (## comments at class level)
    DocComment,

    // 05. signals
    Signal,

    // 06. enums
    Enum,

    // 07. constants
    Const,

    // 08. static variables
    StaticVar,

    // 09. @export variables
    ExportVar,

    // 10. regular variables
    Var,

    // 11. @onready variables
    OnreadyVar,

    // 12. _static_init()
    StaticInit,

    // 13. remaining static methods
    StaticMethod,

    // 14. Virtual methods in specific order
    VirtualInit,
    VirtualEnterTree,
    VirtualReady,
    VirtualProcess,
    VirtualPhysicsProcess,
    VirtualOther,

    // 15. Overridden custom methods (private methods not in virtual list)
    OverriddenCustomMethod,

    // 16. Regular methods
    Method,

    // 17. Inner classes
    InnerClass,
}

impl MemberKind {
    /// Check if this is a header kind (no blank lines between these).
    fn is_header(&self) -> bool {
        matches!(
            self,
            MemberKind::Tool
                | MemberKind::Icon
                | MemberKind::StaticUnload
                | MemberKind::ClassName
                | MemberKind::Extends
        )
    }

    /// Check if this is a function-like kind (2 blank lines around these).
    fn is_function_like(&self) -> bool {
        matches!(
            self,
            MemberKind::StaticInit
                | MemberKind::StaticMethod
                | MemberKind::VirtualInit
                | MemberKind::VirtualEnterTree
                | MemberKind::VirtualReady
                | MemberKind::VirtualProcess
                | MemberKind::VirtualPhysicsProcess
                | MemberKind::VirtualOther
                | MemberKind::OverriddenCustomMethod
                | MemberKind::Method
                | MemberKind::InnerClass
        )
    }
}

/// A declaration with its source text and metadata.
#[derive(Debug, Clone)]
pub struct Declaration {
    /// The kind of member for sorting
    pub kind: MemberKind,

    /// The text of the declaration (including any preceding comments and annotations)
    pub text: String,

    /// Original position for stable sorting
    pub original_index: usize,
}

/// Extract the annotation name from an annotation node.
fn get_annotation_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "identifier" {
            return node_text(child, source);
        }
    }
    None
}

/// Get the text of a node.
fn node_text<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
    source.get(node.start_byte()..node.end_byte())
}

/// Check if a function is static by looking for static keyword.
fn is_static_function(node: Node<'_>) -> bool {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() == "static_keyword" {
            return true;
        }
    }
    false
}

/// Classify a function as a virtual method kind.
fn classify_virtual_method(name: &str) -> MemberKind {
    match name {
        "_init" => MemberKind::VirtualInit,
        "_enter_tree" => MemberKind::VirtualEnterTree,
        "_ready" => MemberKind::VirtualReady,
        "_process" => MemberKind::VirtualProcess,
        "_physics_process" => MemberKind::VirtualPhysicsProcess,
        "_exit_tree" | "_input" | "_unhandled_input" | "_notification" | "_draw" | "_gui_input"
        | "_unhandled_key_input" | "_shortcut_input" | "_get_configuration_warnings"
        | "_get_configuration_warning" => MemberKind::VirtualOther,
        name if name.starts_with('_') => MemberKind::OverriddenCustomMethod,
        _ => MemberKind::Method,
    }
}

/// Check if an annotation is an export variant.
fn is_export_annotation(name: &str) -> bool {
    name == "export" || name.starts_with("export_")
}

/// Check if an annotation is standalone (not attached to a declaration).
fn is_standalone_annotation(name: &str) -> bool {
    matches!(name, "tool" | "icon" | "static_unload")
}

/// Get annotations and modifiers from inside a node.
/// Returns annotations (like @export, @onready) and modifiers (like static).
fn get_node_modifiers(node: Node<'_>, source: &str) -> Vec<String> {
    let mut modifiers = Vec::new();
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "annotations" {
            // Found annotations wrapper - extract individual annotations
            let mut ann_cursor = child.walk();
            for ann in child.children(&mut ann_cursor) {
                if ann.kind() == "annotation" {
                    if let Some(name) = get_annotation_name(ann, source) {
                        modifiers.push(name.to_string());
                    }
                }
            }
        } else if child.kind() == "annotation" {
            // Direct annotation child
            if let Some(name) = get_annotation_name(child, source) {
                modifiers.push(name.to_string());
            }
        } else if child.kind() == "static_keyword" {
            // Static keyword as direct child
            modifiers.push("static".to_string());
        }
    }

    modifiers
}

/// Get the text from a range of lines (1-indexed, inclusive).
fn get_lines_text(source: &str, start_line: usize, end_line: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = String::new();

    for i in (start_line - 1)..end_line.min(lines.len()) {
        result.push_str(lines[i]);
        result.push('\n');
    }

    result
}

/// Extract declarations from a scope.
fn extract_declarations(node: Node<'_>, source: &str, skip_regions: &SkipRegions) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let lines: Vec<&str> = source.lines().collect();

    let mut i = 0;
    let mut original_index = 0;
    let mut processed_annotation_indices = std::collections::HashSet::new();

    while i < children.len() {
        let child = children[i];
        let child_start_line = child.start_position().row + 1;
        let child_end_line = child.end_position().row + 1;

        // Check if in skip region
        if skip_regions.is_skipped(child_start_line) {
            i += 1;
            continue;
        }

        // Handle standalone annotations at top level
        if child.kind() == "annotation" {
            if let Some(name) = get_annotation_name(child, source) {
                if is_standalone_annotation(name) {
                    // Standalone annotation - add as its own declaration
                    let kind = match name {
                        "tool" => MemberKind::Tool,
                        "icon" => MemberKind::Icon,
                        "static_unload" => MemberKind::StaticUnload,
                        _ => unreachable!(),
                    };
                    let text = get_lines_text(source, child_start_line, child_end_line);
                    declarations.push(Declaration {
                        kind,
                        text,
                        original_index,
                    });
                    processed_annotation_indices.insert(i);
                    original_index += 1;
                }
            }
            i += 1;
            continue;
        }

        // Classify based on node type and annotations
        let kind = match child.kind() {
            "class_name_statement" => Some(MemberKind::ClassName),
            "extends_statement" => Some(MemberKind::Extends),
            "signal_statement" => Some(MemberKind::Signal),
            "enum_definition" => Some(MemberKind::Enum),
            "const_statement" => Some(MemberKind::Const),
            "variable_statement" => {
                // Get annotations and modifiers from inside the node (tree-sitter puts them as children)
                let node_modifiers = get_node_modifiers(child, source);

                // Check modifiers (priority: onready > export > static > regular)
                let mut var_kind = MemberKind::Var;
                for m in &node_modifiers {
                    if m == "onready" {
                        var_kind = MemberKind::OnreadyVar;
                        break;
                    }
                }
                if var_kind == MemberKind::Var {
                    for m in &node_modifiers {
                        if is_export_annotation(m) {
                            var_kind = MemberKind::ExportVar;
                            break;
                        }
                    }
                }
                if var_kind == MemberKind::Var {
                    for m in &node_modifiers {
                        if m == "static" {
                            var_kind = MemberKind::StaticVar;
                            break;
                        }
                    }
                }
                Some(var_kind)
            }
            "function_definition" => {
                let is_static = is_static_function(child);
                let name = child
                    .child_by_field_name("name")
                    .and_then(|n| node_text(n, source))
                    .unwrap_or("");

                if is_static {
                    if name == "_static_init" {
                        Some(MemberKind::StaticInit)
                    } else {
                        Some(MemberKind::StaticMethod)
                    }
                } else {
                    Some(classify_virtual_method(name))
                }
            }
            // _init() is parsed as constructor_definition, not function_definition
            "constructor_definition" => Some(MemberKind::VirtualInit),
            "class_definition" => Some(MemberKind::InnerClass),
            "comment" => {
                let text = node_text(child, source).unwrap_or("");
                if text.starts_with("##") {
                    Some(MemberKind::DocComment)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(kind) = kind {
            // Include preceding comments and annotations (no blank line between)
            // This captures @export_category, @export_group, etc. that precede variables
            let mut start_line = child_start_line;
            while start_line > 1 {
                let prev_line_idx = start_line - 2;
                if prev_line_idx >= lines.len() {
                    break;
                }
                let prev_line = lines[prev_line_idx].trim();
                // Include preceding comments (but not doc comments ##)
                if prev_line.starts_with('#') && !prev_line.starts_with("##") {
                    start_line -= 1;
                }
                // Include preceding standalone annotations like @export_category, @export_group
                else if prev_line.starts_with("@export_category")
                    || prev_line.starts_with("@export_group")
                    || prev_line.starts_with("@export_subgroup")
                {
                    start_line -= 1;
                } else {
                    break;
                }
            }

            let text = get_lines_text(source, start_line, child_end_line);

            declarations.push(Declaration {
                kind,
                text,
                original_index,
            });
            original_index += 1;
        }

        i += 1;
    }

    declarations
}

/// Sort declarations by MemberKind, preserving original order within same kind.
fn sort_declarations(declarations: &mut [Declaration]) {
    declarations.sort_by(|a, b| {
        match a.kind.cmp(&b.kind) {
            std::cmp::Ordering::Equal => a.original_index.cmp(&b.original_index),
            other => other,
        }
    });
}

/// Determine blank lines needed between two member kinds.
fn blank_lines_between(prev: MemberKind, next: MemberKind) -> usize {
    // Header items have no blank lines between them
    if prev.is_header() && next.is_header() {
        return 0;
    }

    // Two blank lines before/after functions and classes
    if prev.is_function_like() || next.is_function_like() {
        return 2;
    }

    // Doc comments have 1 blank line after header but before other sections
    if prev == MemberKind::DocComment || next == MemberKind::DocComment {
        return 1;
    }

    // Same category: no blank line
    if prev == next {
        return 0;
    }

    // Different categories: one blank line
    1
}

/// Reconstruct source from sorted declarations.
fn reconstruct_source(declarations: &[Declaration]) -> String {
    if declarations.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let mut prev_kind: Option<MemberKind> = None;

    for decl in declarations {
        // Add appropriate blank lines between sections
        if let Some(pk) = prev_kind {
            let blanks = blank_lines_between(pk, decl.kind);
            for _ in 0..blanks {
                output.push('\n');
            }
        }

        // Add the declaration text (already includes trailing newline)
        output.push_str(&decl.text);

        prev_kind = Some(decl.kind);
    }

    output
}

/// Reorder declarations in source according to GDScript style guide.
pub fn reorder_source(source: &str) -> Result<String, FormatError> {
    if source.trim().is_empty() {
        return Ok(source.to_string());
    }

    let tree = parser::parse(source).map_err(FormatError::Parse)?;
    let root = tree.root_node();
    let skip_regions = SkipRegions::parse(source);

    // Check if any top-level declaration is in a skip region
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        let start_line = child.start_position().row + 1;
        if skip_regions.is_skipped(start_line) {
            return Ok(source.to_string());
        }
    }

    // Extract and sort top-level declarations
    let mut declarations = extract_declarations(root, source, &skip_regions);

    if declarations.is_empty() {
        return Ok(source.to_string());
    }

    // Check if already in correct order
    let original_order: Vec<_> = declarations.iter().map(|d| d.original_index).collect();
    sort_declarations(&mut declarations);
    let sorted_order: Vec<_> = declarations.iter().map(|d| d.original_index).collect();

    // If no reordering needed at top level, check inner classes only
    let top_level_reordered = original_order != sorted_order;

    // Handle inner classes - reorder their bodies
    let mut any_inner_reordered = false;
    for decl in &mut declarations {
        if decl.kind == MemberKind::InnerClass {
            let original = decl.text.clone();
            decl.text = reorder_inner_class(&decl.text, &skip_regions, 1)?;
            if decl.text != original {
                any_inner_reordered = true;
            }
        }
    }

    // If nothing was reordered, return original source to preserve comments
    if !top_level_reordered && !any_inner_reordered {
        return Ok(source.to_string());
    }

    // Reconstruct the source
    let mut result = reconstruct_source(&declarations);

    // Ensure trailing newline
    if !result.ends_with('\n') {
        result.push('\n');
    }

    Ok(result)
}

/// Reorder the body of an inner class.
fn reorder_inner_class(class_text: &str, skip_regions: &SkipRegions, _depth: usize) -> Result<String, FormatError> {
    let tree = parser::parse(class_text).map_err(FormatError::Parse)?;
    let root = tree.root_node();

    // Find the class_definition node
    fn find_class_def(node: Node<'_>) -> Option<Node<'_>> {
        if node.kind() == "class_definition" {
            return Some(node);
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = find_class_def(child) {
                return Some(found);
            }
        }
        None
    }

    let Some(class_node) = find_class_def(root) else {
        return Ok(class_text.to_string());
    };

    let Some(body) = class_node.child_by_field_name("body") else {
        return Ok(class_text.to_string());
    };

    // Get the class header (before the body)
    let header = &class_text[..body.start_byte()];

    // Get body content
    let body_text = &class_text[body.start_byte()..body.end_byte()];

    // Parse the body to extract declarations
    let body_tree = parser::parse(body_text).map_err(FormatError::Parse)?;
    let body_root = body_tree.root_node();

    let mut declarations = extract_declarations(body_root, body_text, skip_regions);

    if declarations.is_empty() {
        return Ok(class_text.to_string());
    }

    sort_declarations(&mut declarations);

    // Recursively handle nested inner classes
    for decl in &mut declarations {
        if decl.kind == MemberKind::InnerClass {
            decl.text = reorder_inner_class(&decl.text, skip_regions, _depth + 1)?;
        }
    }

    // Reconstruct the body - preserve original indentation from declarations
    let mut output = String::new();
    output.push_str(header);
    output.push('\n'); // Newline after header

    let mut prev_kind: Option<MemberKind> = None;

    for decl in &declarations {
        if let Some(pk) = prev_kind {
            let blanks = blank_lines_between(pk, decl.kind);
            for _ in 0..blanks {
                output.push('\n');
            }
        }

        // Preserve the declaration text as-is (already has proper indentation)
        output.push_str(&decl.text);

        prev_kind = Some(decl.kind);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_kind_ordering() {
        assert!(MemberKind::Tool < MemberKind::ClassName);
        assert!(MemberKind::ClassName < MemberKind::Extends);
        assert!(MemberKind::Extends < MemberKind::Signal);
        assert!(MemberKind::Signal < MemberKind::Enum);
        assert!(MemberKind::Enum < MemberKind::Const);
        assert!(MemberKind::Const < MemberKind::StaticVar);
        assert!(MemberKind::StaticVar < MemberKind::ExportVar);
        assert!(MemberKind::ExportVar < MemberKind::Var);
        assert!(MemberKind::Var < MemberKind::OnreadyVar);
        assert!(MemberKind::OnreadyVar < MemberKind::StaticInit);
        assert!(MemberKind::StaticInit < MemberKind::StaticMethod);
        assert!(MemberKind::StaticMethod < MemberKind::VirtualInit);
        assert!(MemberKind::VirtualInit < MemberKind::VirtualEnterTree);
        assert!(MemberKind::VirtualEnterTree < MemberKind::VirtualReady);
        assert!(MemberKind::VirtualReady < MemberKind::VirtualProcess);
        assert!(MemberKind::VirtualProcess < MemberKind::VirtualPhysicsProcess);
        assert!(MemberKind::VirtualPhysicsProcess < MemberKind::VirtualOther);
        assert!(MemberKind::VirtualOther < MemberKind::OverriddenCustomMethod);
        assert!(MemberKind::OverriddenCustomMethod < MemberKind::Method);
        assert!(MemberKind::Method < MemberKind::InnerClass);
    }

    #[test]
    fn test_is_export_annotation() {
        assert!(is_export_annotation("export"));
        assert!(is_export_annotation("export_range"));
        assert!(is_export_annotation("export_enum"));
        assert!(!is_export_annotation("onready"));
        assert!(!is_export_annotation("tool"));
    }

    #[test]
    fn test_classify_virtual_method() {
        assert_eq!(classify_virtual_method("_init"), MemberKind::VirtualInit);
        assert_eq!(classify_virtual_method("_ready"), MemberKind::VirtualReady);
        assert_eq!(
            classify_virtual_method("_enter_tree"),
            MemberKind::VirtualEnterTree
        );
        assert_eq!(
            classify_virtual_method("_process"),
            MemberKind::VirtualProcess
        );
        assert_eq!(
            classify_virtual_method("_physics_process"),
            MemberKind::VirtualPhysicsProcess
        );
        assert_eq!(
            classify_virtual_method("_exit_tree"),
            MemberKind::VirtualOther
        );
        assert_eq!(
            classify_virtual_method("_custom"),
            MemberKind::OverriddenCustomMethod
        );
        assert_eq!(classify_virtual_method("foo"), MemberKind::Method);
    }

    #[test]
    fn test_is_standalone_annotation() {
        assert!(is_standalone_annotation("tool"));
        assert!(is_standalone_annotation("icon"));
        assert!(is_standalone_annotation("static_unload"));
        assert!(!is_standalone_annotation("export"));
        assert!(!is_standalone_annotation("onready"));
    }
}
