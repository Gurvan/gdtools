//! Code reordering according to the GDScript style guide.
//!
//! This module reorders class members to follow the official ordering:
//! 1. @tool, @icon, @static_unload
//! 2. class_name
//! 3. extends
//! 4. signals
//! 5. enums
//! 6. constants
//! 7. static variables
//! 8. @export variables
//! 9. remaining regular variables
//! 10. @onready variables
//! 11. _static_init()
//! 12. remaining static methods
//! 13. virtual methods (_init, _enter_tree, _ready, _process, _physics_process, others)
//! 14. overridden custom methods
//! 15. remaining methods
//! 16. subclasses
//!
//! Comments (including ## doc comments) are attached to the following declaration
//! and move with it during reordering.

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

    // 04. signals
    Signal,

    // 05. enums
    Enum,

    // 06. constants
    Const,

    // 07. static variables
    StaticVar,

    // 08. @export variables
    ExportVar,

    // 09. regular variables
    Var,

    // 10. @onready variables
    OnreadyVar,

    // 11. _static_init()
    StaticInit,

    // 12. remaining static methods
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

    /// Whether this declaration has a leading doc comment (##)
    pub has_doc_comment: bool,

    /// Whether this declaration has a leading section annotation (@export_category, @export_group, @export_subgroup)
    pub has_section_annotation: bool,
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
        "_exit_tree"
        | "_input"
        | "_unhandled_input"
        | "_notification"
        | "_draw"
        | "_gui_input"
        | "_unhandled_key_input"
        | "_shortcut_input"
        | "_get_configuration_warnings"
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

/// Check if an annotation is a section marker (export_category, export_group, export_subgroup).
fn is_section_annotation(name: &str) -> bool {
    matches!(name, "export_category" | "export_group" | "export_subgroup")
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
fn extract_declarations(
    node: Node<'_>,
    source: &str,
    skip_regions: &SkipRegions,
) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

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
                        has_doc_comment: false,
                        has_section_annotation: false,
                    });
                    processed_annotation_indices.insert(i);
                    original_index += 1;
                } else if is_section_annotation(name) {
                    // Check if this section annotation is followed by a variable
                    // If not, it's orphaned and should be preserved as-is
                    let mut has_following_var = false;
                    for j in (i + 1)..children.len() {
                        let next_child = children[j];
                        if next_child.kind() == "variable_statement" {
                            has_following_var = true;
                            break;
                        }
                        // Stop if we hit another major declaration type
                        if matches!(
                            next_child.kind(),
                            "function_definition"
                                | "class_definition"
                                | "signal_statement"
                                | "enum_definition"
                                | "const_statement"
                        ) {
                            break;
                        }
                    }
                    if !has_following_var {
                        // Orphaned section annotation - preserve it with the last variable kind
                        // Also look for preceding comments to include with it
                        let mut start_line = child_start_line;
                        let mut has_doc_comment = false;

                        // Look backwards for preceding comments
                        let mut prev_idx = i;
                        while prev_idx > 0 {
                            prev_idx -= 1;
                            let prev_child = children[prev_idx];

                            // Skip if already processed
                            if processed_annotation_indices.contains(&prev_idx) {
                                break;
                            }

                            // Check for comments
                            if prev_child.kind() == "comment" {
                                // Only consider comments that start at the beginning of a line
                                if prev_child.start_position().column != 0 {
                                    break;
                                }
                                let prev_end_line = prev_child.end_position().row + 1;
                                // Allow up to 1 blank line between comment and annotation
                                let is_near = prev_end_line + 2 >= start_line;

                                if is_near {
                                    // Check if it's a doc comment
                                    let is_contiguous = prev_end_line + 1 >= start_line;
                                    if is_contiguous {
                                        if let Some(text) = node_text(prev_child, source) {
                                            if text.starts_with("##") {
                                                has_doc_comment = true;
                                            }
                                        }
                                    }
                                    start_line = prev_child.start_position().row + 1;
                                    processed_annotation_indices.insert(prev_idx);
                                    continue;
                                }
                            }
                            break;
                        }

                        let text = get_lines_text(source, start_line, child_end_line);
                        declarations.push(Declaration {
                            kind: MemberKind::Var,
                            text,
                            original_index,
                            has_doc_comment,
                            has_section_annotation: true,
                        });
                        processed_annotation_indices.insert(i);
                        original_index += 1;
                    }
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
            // Comments (including ## doc comments) are not standalone declarations.
            // They are included with the following declaration they document.
            _ => None,
        };

        if let Some(kind) = kind {
            // Include preceding comments and annotations (no blank line between)
            // This captures @export_category, @export_group, etc. that precede variables
            let mut start_line = child_start_line;
            let mut has_doc_comment = false;
            let mut has_section_annotation = false;

            // First, look at preceding AST children for annotations like @export_category
            // This handles multiline annotations correctly by using AST node boundaries
            let mut prev_idx = i;
            while prev_idx > 0 {
                prev_idx -= 1;
                let prev_child = children[prev_idx];

                // Skip if already processed as a standalone annotation
                if processed_annotation_indices.contains(&prev_idx) {
                    break;
                }

                // Check for annotation nodes
                if prev_child.kind() == "annotation" {
                    if let Some(name) = get_annotation_name(prev_child, source) {
                        // Check if it's an export-related annotation that should move with the variable
                        // This includes: @export, @export_category, @export_group, @export_subgroup,
                        // @export_enum, @export_flags, @export_range, @export_multiline, etc.
                        let is_export_annotation = name == "export" || name.starts_with("export_");
                        if is_export_annotation && !is_standalone_annotation(name) {
                            let prev_end_line = prev_child.end_position().row + 1;
                            // Check if contiguous (no blank line between)
                            if prev_end_line + 1 >= start_line {
                                // Check if it's a section annotation
                                if is_section_annotation(name) {
                                    has_section_annotation = true;
                                }
                                start_line = prev_child.start_position().row + 1;
                                processed_annotation_indices.insert(prev_idx);
                                continue;
                            }
                        }
                    }
                }
                // Check for comments (including ## doc comments)
                // Comments immediately preceding a declaration are attached to it
                // But NOT inline/trailing comments (those that start in the middle of a line)
                else if prev_child.kind() == "comment" {
                    // Only consider comments that start at the beginning of a line (column 0)
                    // This excludes inline comments like `var x = 1 # comment`
                    if prev_child.start_position().column != 0 {
                        break;
                    }
                    let prev_end_line = prev_child.end_position().row + 1;
                    // Include comments that precede a declaration, allowing:
                    // - Up to 2 blank lines for functions/classes (for section headers)
                    // - Up to 1 blank line for other declarations
                    let is_contiguous = prev_end_line + 1 >= start_line;
                    let is_near = prev_end_line + 2 >= start_line;
                    let is_header_comment =
                        kind.is_function_like() && prev_end_line + 3 >= start_line;

                    if is_contiguous || is_near || is_header_comment {
                        // Check if it's a doc comment (only if immediately preceding)
                        if is_contiguous {
                            if let Some(text) = node_text(prev_child, source) {
                                if text.starts_with("##") {
                                    has_doc_comment = true;
                                }
                            }
                        }
                        start_line = prev_child.start_position().row + 1;
                        processed_annotation_indices.insert(prev_idx);
                        continue;
                    }
                }
                break;
            }

            let text = get_lines_text(source, start_line, child_end_line);

            declarations.push(Declaration {
                kind,
                text,
                original_index,
                has_doc_comment,
                has_section_annotation,
            });
            original_index += 1;
        }

        i += 1;
    }

    declarations
}

/// Sort declarations by MemberKind, preserving original order within same kind.
fn sort_declarations(declarations: &mut [Declaration]) {
    declarations.sort_by(|a, b| match a.kind.cmp(&b.kind) {
        std::cmp::Ordering::Equal => a.original_index.cmp(&b.original_index),
        other => other,
    });
}

/// Determine blank lines needed between two declarations.
fn blank_lines_between(prev: &Declaration, next: &Declaration) -> usize {
    // Header items have no blank lines between them
    if prev.kind.is_header() && next.kind.is_header() {
        return 0;
    }

    // Two blank lines before/after functions and classes
    if prev.kind.is_function_like() || next.kind.is_function_like() {
        return 2;
    }

    // If next declaration has a doc comment or section annotation, add a blank line before it
    // This keeps doc-commented and @export_category/@export_group sections visually separated
    if next.has_doc_comment || next.has_section_annotation {
        return 1;
    }

    // Same category: no blank line
    if prev.kind == next.kind {
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
    let mut prev_decl: Option<&Declaration> = None;

    for decl in declarations {
        // Add appropriate blank lines between sections
        if let Some(prev) = prev_decl {
            let blanks = blank_lines_between(prev, decl);
            for _ in 0..blanks {
                output.push('\n');
            }
        }

        // Add the declaration text (already includes trailing newline)
        output.push_str(&decl.text);

        prev_decl = Some(decl);
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
fn reorder_inner_class(
    class_text: &str,
    skip_regions: &SkipRegions,
    _depth: usize,
) -> Result<String, FormatError> {
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

    let mut prev_decl: Option<&Declaration> = None;

    for decl in &declarations {
        if let Some(prev) = prev_decl {
            let blanks = blank_lines_between(prev, decl);
            for _ in 0..blanks {
                output.push('\n');
            }
        }

        // Preserve the declaration text as-is (already has proper indentation)
        output.push_str(&decl.text);

        prev_decl = Some(decl);
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

    #[test]
    fn test_debug_multiline_export_category() {
        let source = r#"extends ActionProperties

class_name TurnProperties

@export_category("Turn Properties")
@export_category("tooltip:Number of frames for slow turn to reverse the fighter. " +
	"On smash turn, value is ignored as fighter reverses instantly.")
var reverse_direction_frame: float
"#;
        let tree = crate::parser::parse(source).unwrap();
        let root = tree.root_node();

        println!("=== AST structure for multiline export_category ===");
        fn print_node(node: tree_sitter::Node, source: &str, depth: usize) {
            let indent = "  ".repeat(depth);
            let text = &source[node.start_byte()..node.end_byte()];
            let text_short = text
                .replace('\n', "\\n")
                .replace('\t', "\\t")
                .chars()
                .take(80)
                .collect::<String>();
            println!(
                "{}kind={:?} lines={}-{} text={:?}",
                indent,
                node.kind(),
                node.start_position().row + 1,
                node.end_position().row + 1,
                text_short
            );

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                print_node(child, source, depth + 1);
            }
        }
        print_node(root, source, 0);
        println!("==================");
    }
}
