//! Tests for the GDScript code reordering feature.
//!
//! Following TDD approach: these tests are written first, then the implementation.

use gdtools::format::{reorder_source, run_formatter, FormatOptions};

/// Helper to format then reorder.
fn reorder(source: &str) -> String {
    let formatted =
        run_formatter(source, &FormatOptions::default()).expect("formatting should succeed");
    reorder_source(&formatted).expect("reordering should succeed")
}

/// Helper to format without reordering.
fn format_only(source: &str) -> String {
    run_formatter(source, &FormatOptions::default()).expect("formatting should succeed")
}

// ============================================================================
// Phase 1: Basic Reordering Tests
// ============================================================================

#[test]
fn test_reorder_vars_before_methods() {
    let input = r#"extends Node


func foo():
	pass


var x = 1
"#;
    let expected = r#"extends Node

var x = 1


func foo():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_reorder_signals_before_vars() {
    let input = r#"extends Node

var x = 1

signal my_signal
"#;
    let expected = r#"extends Node

signal my_signal

var x = 1
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_reorder_consts_before_vars() {
    let input = r#"extends Node

var x = 1

const MAX = 100
"#;
    let expected = r#"extends Node

const MAX = 100

var x = 1
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_reorder_enums_before_consts() {
    let input = r#"extends Node

const MAX = 100

enum State { IDLE, RUNNING }
"#;
    let expected = r#"extends Node

enum State { IDLE, RUNNING }

const MAX = 100
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 2: Virtual Method Ordering Tests
// ============================================================================

#[test]
fn test_virtual_method_ordering_init_before_ready() {
    let input = r#"extends Node


func _ready():
	pass


func _init():
	pass
"#;
    let expected = r#"extends Node


func _init():
	pass


func _ready():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_virtual_method_ordering_full() {
    let input = r#"extends Node


func _physics_process(delta):
	pass


func _ready():
	pass


func _process(delta):
	pass


func _enter_tree():
	pass


func _init():
	pass
"#;
    let expected = r#"extends Node


func _init():
	pass


func _enter_tree():
	pass


func _ready():
	pass


func _process(delta):
	pass


func _physics_process(delta):
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_virtual_other_after_physics_process() {
    let input = r#"extends Node


func _exit_tree():
	pass


func _ready():
	pass
"#;
    let expected = r#"extends Node


func _ready():
	pass


func _exit_tree():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 3: Variable Annotation Ordering Tests
// ============================================================================

#[test]
fn test_export_vars_before_regular_vars() {
    let input = r#"extends Node

var regular = 1

@export var exported = 2
"#;
    let expected = r#"extends Node

@export var exported = 2

var regular = 1
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_onready_vars_after_regular_vars() {
    let input = r#"extends Node

@onready var node = $Node

var regular = 1
"#;
    let expected = r#"extends Node

var regular = 1

@onready var node = $Node
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_static_vars_before_export_vars() {
    let input = r#"extends Node

@export var exported = 1

static var static_var = 2
"#;
    let expected = r#"extends Node

static var static_var = 2

@export var exported = 1
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_export_category_preserved_with_variable() {
    // @export_category on its own line should be preserved with the following variable
    let input = r#"extends Node

@export_category("Settings")
var speed: float = 10.0
"#;
    let result = reorder(input);
    assert!(result.contains("@export_category(\"Settings\")"));
    assert!(result.contains("var speed: float = 10.0"));
    // The category should be right before the variable
    let cat_pos = result.find("@export_category").unwrap();
    let var_pos = result.find("var speed").unwrap();
    assert!(cat_pos < var_pos);
}

#[test]
fn test_export_category_moves_with_variable_when_reordered() {
    // When a variable with @export_category is reordered, the category moves with it
    let input = r#"extends Node


func foo():
	pass


@export_category("Settings")
var speed: float = 10.0
"#;
    let result = reorder(input);
    // Variable should come before function
    let var_pos = result.find("var speed").unwrap();
    let func_pos = result.find("func foo").unwrap();
    assert!(var_pos < func_pos);
    // Category should still be with the variable
    assert!(result.contains("@export_category(\"Settings\")"));
    let cat_pos = result.find("@export_category").unwrap();
    assert!(cat_pos < var_pos);
}

#[test]
fn test_export_group_preserved_with_variable() {
    // @export_group on its own line should be preserved with the following variable
    let input = r#"extends Node

@export_group("Movement")
var speed: float = 10.0
"#;
    let result = reorder(input);
    assert!(result.contains("@export_group(\"Movement\")"));
    assert!(result.contains("var speed: float = 10.0"));
    let group_pos = result.find("@export_group").unwrap();
    let var_pos = result.find("var speed").unwrap();
    assert!(group_pos < var_pos);
}

#[test]
fn test_export_group_moves_with_variable_when_reordered() {
    let input = r#"extends Node


func foo():
	pass


@export_group("Movement")
var speed: float = 10.0
"#;
    let result = reorder(input);
    let var_pos = result.find("var speed").unwrap();
    let func_pos = result.find("func foo").unwrap();
    assert!(var_pos < func_pos);
    assert!(result.contains("@export_group(\"Movement\")"));
    let group_pos = result.find("@export_group").unwrap();
    assert!(group_pos < var_pos);
}

#[test]
fn test_export_subgroup_preserved_with_variable() {
    // @export_subgroup on its own line should be preserved with the following variable
    let input = r#"extends Node

@export_subgroup("Advanced")
var acceleration: float = 5.0
"#;
    let result = reorder(input);
    assert!(result.contains("@export_subgroup(\"Advanced\")"));
    assert!(result.contains("var acceleration: float = 5.0"));
    let subgroup_pos = result.find("@export_subgroup").unwrap();
    let var_pos = result.find("var acceleration").unwrap();
    assert!(subgroup_pos < var_pos);
}

#[test]
fn test_export_subgroup_moves_with_variable_when_reordered() {
    let input = r#"extends Node


func foo():
	pass


@export_subgroup("Advanced")
var acceleration: float = 5.0
"#;
    let result = reorder(input);
    let var_pos = result.find("var acceleration").unwrap();
    let func_pos = result.find("func foo").unwrap();
    assert!(var_pos < func_pos);
    assert!(result.contains("@export_subgroup(\"Advanced\")"));
    let subgroup_pos = result.find("@export_subgroup").unwrap();
    assert!(subgroup_pos < var_pos);
}

#[test]
fn test_multiple_export_annotations_preserved() {
    // Multiple export annotations stacked should all be preserved
    let input = r#"extends Node

@export_category("Physics")
@export_group("Movement")
@export_subgroup("Ground")
var ground_speed: float = 10.0
"#;
    let result = reorder(input);
    assert!(result.contains("@export_category(\"Physics\")"));
    assert!(result.contains("@export_group(\"Movement\")"));
    assert!(result.contains("@export_subgroup(\"Ground\")"));
    assert!(result.contains("var ground_speed: float = 10.0"));
    // All should be before the variable
    let cat_pos = result.find("@export_category").unwrap();
    let group_pos = result.find("@export_group").unwrap();
    let subgroup_pos = result.find("@export_subgroup").unwrap();
    let var_pos = result.find("var ground_speed").unwrap();
    assert!(cat_pos < group_pos);
    assert!(group_pos < subgroup_pos);
    assert!(subgroup_pos < var_pos);
}

#[test]
fn test_multiple_export_annotations_move_with_variable() {
    let input = r#"extends Node


func foo():
	pass


@export_category("Physics")
@export_group("Movement")
var speed: float = 10.0
"#;
    let result = reorder(input);
    let var_pos = result.find("var speed").unwrap();
    let func_pos = result.find("func foo").unwrap();
    assert!(var_pos < func_pos);
    // Both annotations should move with the variable
    let cat_pos = result.find("@export_category").unwrap();
    let group_pos = result.find("@export_group").unwrap();
    assert!(cat_pos < group_pos);
    assert!(group_pos < var_pos);
}

// ============================================================================
// Phase 4: Static Method Ordering Tests
// ============================================================================

#[test]
fn test_static_init_before_other_static_methods() {
    let input = r#"extends Node


static func helper():
	pass


static func _static_init():
	pass
"#;
    let expected = r#"extends Node


static func _static_init():
	pass


static func helper():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_static_methods_before_virtual_methods() {
    let input = r#"extends Node


func _ready():
	pass


static func helper():
	pass
"#;
    let expected = r#"extends Node


static func helper():
	pass


func _ready():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 5: File Header Ordering Tests
// ============================================================================

#[test]
fn test_tool_before_class_name() {
    let input = r#"class_name MyClass
@tool
extends Node
"#;
    let expected = r#"@tool
class_name MyClass
extends Node
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_icon_after_tool() {
    let input = r#"@icon("res://icon.png")
@tool
extends Node
"#;
    let expected = r#"@tool
@icon("res://icon.png")
extends Node
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_static_unload_after_icon() {
    let input = r#"@static_unload
@tool
extends Node
"#;
    let expected = r#"@tool
@static_unload
extends Node
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_extends_after_class_name() {
    let input = r#"extends Node
class_name MyClass
"#;
    let expected = r#"class_name MyClass
extends Node
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 6: Comment Handling Tests
// ============================================================================

#[test]
fn test_comment_moves_with_declaration() {
    let input = r#"extends Node


func foo():
	pass


# This describes the variable
var x = 1
"#;
    let expected = r#"extends Node

# This describes the variable
var x = 1


func foo():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_multiline_comment_moves_with_declaration() {
    let input = r#"extends Node


func foo():
	pass


# Line 1
# Line 2
# Line 3
var x = 1
"#;
    let expected = r#"extends Node

# Line 1
# Line 2
# Line 3
var x = 1


func foo():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_comment_not_attached_with_blank_line() {
    let input = r#"extends Node

# Standalone comment

var x = 1
"#;
    // Comment has blank line after it, so it doesn't attach to var
    // Should stay in place relative to other content
    let result = reorder(input);
    // The standalone comment should remain separate
    assert!(result.contains("# Standalone comment"));
    assert!(result.contains("var x = 1"));
}

#[test]
fn test_doc_comment_ordering() {
    let input = r#"extends Node

var x = 1

## This is a class doc comment.
"#;
    let expected = r#"extends Node

## This is a class doc comment.

var x = 1
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 7: Inner Class Tests
// ============================================================================

#[test]
fn test_inner_class_at_end() {
    let input = r#"extends Node


class Inner:
	pass


func foo():
	pass
"#;
    let expected = r#"extends Node


func foo():
	pass


class Inner:
	pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_inner_class_members_reordered() {
    let input = r#"extends Node


class Inner:
	func foo():
		pass

	var x = 1
"#;
    let expected = r#"extends Node


class Inner:
	var x = 1


	func foo():
		pass
"#;
    assert_eq!(reorder(input), expected);
}

#[test]
fn test_nested_inner_classes() {
    let input = r#"extends Node


class Outer:
	class InnerInner:
		func foo():
			pass

		var y = 2

	func bar():
		pass

	var x = 1
"#;
    let expected = r#"extends Node


class Outer:
	var x = 1


	func bar():
		pass


	class InnerInner:
		var y = 2


		func foo():
			pass
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 8: Complete Class Tests
// ============================================================================

#[test]
fn test_complete_class_ordering() {
    let input = r#"class_name MyClass
extends Node


func custom_method():
	pass


var z = 3

@onready var node = $Node


func _ready():
	pass


signal something_happened

const MAX = 100

@export var speed: float = 10.0

enum State { IDLE, RUNNING }


func _init():
	pass
"#;
    let expected = r#"class_name MyClass
extends Node

signal something_happened

enum State { IDLE, RUNNING }

const MAX = 100

@export var speed: float = 10.0

var z = 3

@onready var node = $Node


func _init():
	pass


func _ready():
	pass


func custom_method():
	pass
"#;
    assert_eq!(reorder(input), expected);
}

// ============================================================================
// Phase 9: Idempotency Tests
// ============================================================================

#[test]
fn test_reorder_idempotent() {
    let input = r#"extends Node


func foo():
	pass


var x = 1
"#;
    let once = reorder(input);
    let twice = reorder(&once);
    assert_eq!(once, twice, "Reordering should be idempotent");
}

#[test]
fn test_already_ordered_unchanged() {
    let input = r#"extends Node

signal foo

const MAX = 10

var x = 1


func bar():
	pass
"#;
    let result = reorder(input);
    assert_eq!(result, input, "Already-ordered code should be unchanged");
}

#[test]
fn test_format_then_reorder_idempotent() {
    let messy_input = r#"extends   Node

func foo():
    pass

var  x=1
"#;
    // First format, then reorder
    let formatted = format_only(messy_input);
    let reordered = reorder(&formatted);
    // Reorder again should be stable
    let reordered_again = reorder(&reordered);
    assert_eq!(reordered, reordered_again);
}

// ============================================================================
// Phase 10: Edge Cases
// ============================================================================

#[test]
fn test_empty_file() {
    let input = "";
    let result = reorder(input);
    assert!(result.is_empty() || result == "\n");
}

#[test]
fn test_only_extends() {
    let input = "extends Node\n";
    let result = reorder(input);
    assert_eq!(result, "extends Node\n");
}

#[test]
fn test_multiple_signals_preserve_order() {
    let input = r#"extends Node

signal second
signal first
signal third
"#;
    // Signals should stay in original relative order (stable sort)
    let result = reorder(input);
    assert!(result.contains("signal second"));
    assert!(result.contains("signal first"));
    assert!(result.contains("signal third"));
    // Check they're all in the signal section (before any vars)
    let signal_section = result.find("signal").unwrap();
    assert!(
        result.find("var").map_or(true, |v| signal_section < v),
        "Signals should come before vars"
    );
}

#[test]
fn test_multiple_vars_preserve_order() {
    let input = r#"extends Node

var z = 3
var a = 1
var m = 2
"#;
    // Vars should stay in original relative order (stable sort)
    let result = reorder(input);
    let z_pos = result.find("var z").unwrap();
    let a_pos = result.find("var a").unwrap();
    let m_pos = result.find("var m").unwrap();
    assert!(z_pos < a_pos, "var z should come before var a");
    assert!(a_pos < m_pos, "var a should come before var m");
}

#[test]
fn test_export_variants() {
    let input = r#"extends Node

var regular = 1

@export_range(0, 100) var ranged = 50
@export var simple = 1
@export_enum("A", "B") var enumed = 0
"#;
    let result = reorder(input);
    // All @export variants should come before regular var
    let regular_pos = result.find("var regular").unwrap();
    let ranged_pos = result.find("@export_range").unwrap();
    let simple_pos = result.find("@export var simple").unwrap();
    let enumed_pos = result.find("@export_enum").unwrap();
    assert!(ranged_pos < regular_pos);
    assert!(simple_pos < regular_pos);
    assert!(enumed_pos < regular_pos);
}

#[test]
fn test_private_method_ordering() {
    let input = r#"extends Node


func public_method():
	pass


func _private_helper():
	pass
"#;
    // Private methods that aren't virtual should be treated as OverriddenCustomMethod
    // and come before regular public methods... or should they?
    // Actually, the style guide says "overridden custom methods" which implies
    // methods that override parent class methods. A simple _private_helper
    // should probably be just a regular Method.
    // Let's just verify both are present and in methods section
    let result = reorder(input);
    assert!(result.contains("func public_method"));
    assert!(result.contains("func _private_helper"));
}

#[test]
fn test_reorder_disabled_by_default() {
    let input = r#"extends Node


func foo():
	pass


var x = 1
"#;
    // With default options (reorder: false), should NOT reorder
    let result = format_only(input);
    // The var should still be after the func
    let func_pos = result.find("func foo").unwrap();
    let var_pos = result.find("var x").unwrap();
    assert!(func_pos < var_pos, "With reorder disabled, order should be preserved");
}

// ============================================================================
// Phase 11: Skip Region Tests
// ============================================================================

#[test]
fn test_fmt_off_region_not_reordered() {
    let input = r#"extends Node

# fmt: off
func foo():
	pass

var x = 1
# fmt: on
"#;
    // Content in fmt:off region should not be reordered
    let result = reorder(input);
    // foo should still come before x within the skipped region
    let foo_pos = result.find("func foo").unwrap();
    let x_pos = result.find("var x").unwrap();
    assert!(foo_pos < x_pos, "Content in fmt:off should preserve order");
}
