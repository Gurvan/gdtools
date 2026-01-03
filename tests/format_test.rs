use gdlint::format::{compare_ast_with_source, run_formatter, AstCheckResult, FormatOptions};
use tree_sitter::Parser;

fn format(source: &str) -> String {
    run_formatter(source, &FormatOptions::default()).unwrap()
}

fn format_with_spaces(source: &str, spaces: usize) -> String {
    run_formatter(source, &FormatOptions::with_spaces(spaces)).unwrap()
}

// Helper to check formatting doesn't crash and produces valid output
fn format_ok(source: &str) -> bool {
    run_formatter(source, &FormatOptions::default()).is_ok()
}

fn parse(source: &str) -> tree_sitter::Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_gdscript::LANGUAGE.into())
        .unwrap();
    parser.parse(source, None).unwrap()
}

/// Assert that formatting preserves AST equivalence.
/// This is the key invariant: formatting should not change the meaning of code.
fn assert_ast_equivalent(source: &str) {
    let formatted = format(source);
    let original_tree = parse(source);
    let formatted_tree = parse(&formatted);

    let result = compare_ast_with_source(&original_tree, source, &formatted_tree, &formatted);
    match result {
        AstCheckResult::Equivalent => {}
        AstCheckResult::Different { path, difference } => {
            panic!(
                "AST changed after formatting!\n\
                 Path: {}\n\
                 Difference: {}\n\
                 Original:\n{}\n\
                 Formatted:\n{}",
                path, difference, source, formatted
            );
        }
    }
}

#[test]
fn test_extends_statement() {
    assert_eq!(format("extends  Node2D\n"), "extends Node2D\n");
    assert_eq!(format("extends   Node2D\n"), "extends Node2D\n");
}

#[test]
fn test_class_name_statement() {
    assert_eq!(format("class_name   MyClass\n"), "class_name MyClass\n");
}

#[test]
fn test_variable_statement() {
    assert_eq!(format("var x:int=1\n"), "var x: int = 1\n");
    assert_eq!(format("var   x  :  int   =   1\n"), "var x: int = 1\n");
    assert_eq!(format("var x = 1\n"), "var x = 1\n");
}

#[test]
fn test_inferred_type_variable() {
    // := is inferred type assignment
    assert_eq!(format("var x := 1\n"), "var x := 1\n");
    assert_eq!(format("var x:=1\n"), "var x := 1\n");
    assert_eq!(format("var gltf := GLTFDocument.new()\n"), "var gltf := GLTFDocument.new()\n");
}

#[test]
fn test_const_statement() {
    assert_eq!(format("const X:int=1\n"), "const X: int = 1\n");
    assert_eq!(format("const   X   =   100\n"), "const X = 100\n");
}

#[test]
fn test_function_definition() {
    assert_eq!(
        format("func foo(  ):\n\tpass\n"),
        "func foo():\n\tpass\n"
    );
    assert_eq!(
        format("func foo(a:int,b:int)->int:\n\treturn a+b\n"),
        "func foo(a: int, b: int) -> int:\n\treturn a + b\n"
    );
}

#[test]
fn test_function_call() {
    assert_eq!(format("print(   \"Hello\"   )\n"), "print(\"Hello\")\n");
    assert_eq!(format("foo(  a  ,  b  ,  c  )\n"), "foo(a, b, c)\n");
}

#[test]
fn test_binary_operators() {
    assert_eq!(format("var x = a+b\n"), "var x = a + b\n");
    assert_eq!(format("var x = a-b\n"), "var x = a - b\n");
    assert_eq!(format("var x = a*b\n"), "var x = a * b\n");
    assert_eq!(format("var x = a/b\n"), "var x = a / b\n");
}

#[test]
fn test_return_statement() {
    assert_eq!(format("func foo():\n\treturn\n"), "func foo():\n\treturn\n");
    assert_eq!(
        format("func foo():\n\treturn a+b\n"),
        "func foo():\n\treturn a + b\n"
    );
}

#[test]
fn test_pass_break_continue() {
    assert_eq!(format("func foo():\n\tpass\n"), "func foo():\n\tpass\n");
    assert_eq!(
        format("func foo():\n\twhile true:\n\t\tbreak\n"),
        "func foo():\n\twhile true:\n\t\tbreak\n"
    );
    assert_eq!(
        format("func foo():\n\twhile true:\n\t\tcontinue\n"),
        "func foo():\n\twhile true:\n\t\tcontinue\n"
    );
}

#[test]
fn test_fmt_off_on() {
    let source = "extends Node2D\n# fmt: off\nvar   x   =   1\n# fmt: on\nvar y = 2\n";
    let formatted = format(source);
    // The fmt:off region should be preserved
    assert!(formatted.contains("var   x   =   1"));
    // The rest should be formatted
    assert!(formatted.contains("var y = 2"));
}

#[test]
fn test_indent_with_spaces() {
    let source = "func foo():\n\tpass\n";
    let formatted = format_with_spaces(source, 4);
    assert_eq!(formatted, "func foo():\n    pass\n");
}

#[test]
fn test_trailing_newline() {
    assert_eq!(format("var x = 1"), "var x = 1\n");
    assert_eq!(format("var x = 1\n"), "var x = 1\n");
    assert_eq!(format("var x = 1\n\n"), "var x = 1\n");
}

#[test]
fn test_array_literal() {
    assert_eq!(format("var x = [1,2,3]\n"), "var x = [1, 2, 3]\n");
    assert_eq!(format("var x = [  1  ,  2  ,  3  ]\n"), "var x = [1, 2, 3]\n");
    assert_eq!(format("var x = []\n"), "var x = []\n");
}

#[test]
fn test_dictionary_literal() {
    // TODO: Dictionary pair formatting needs more investigation of tree-sitter node types
    // For now, just verify it doesn't crash and empty dict works
    let _ = format("var x = {a:1,b:2}\n");
    assert_eq!(format("var x = {}\n"), "var x = {}\n");
}

#[test]
fn test_if_statement() {
    let source = "if x==1:\n\tpass\n";
    let formatted = format(source);
    assert!(formatted.contains("if x == 1:"));
}

#[test]
fn test_for_statement() {
    let source = "for i in range(10):\n\tpass\n";
    let formatted = format(source);
    assert!(formatted.contains("for i in range(10):"));
}

#[test]
fn test_while_statement() {
    let source = "while x>0:\n\tpass\n";
    let formatted = format(source);
    assert!(formatted.contains("while x > 0:"));
}

#[test]
fn test_signal_statement() {
    assert_eq!(format("signal my_signal\n"), "signal my_signal\n");
}

#[test]
fn test_enum_definition() {
    assert_eq!(
        format("enum State { IDLE, WALKING, RUNNING }\n"),
        "enum State { IDLE, WALKING, RUNNING }\n"
    );
}

#[test]
fn test_function_default_parameters() {
    // Default parameter values should be preserved
    assert!(format_ok("func foo(x: int = 5):\n\tpass\n"));
    assert!(format_ok("func foo(_with_model: bool = false):\n\tpass\n"));
}

#[test]
fn test_method_chaining() {
    assert_eq!(
        format("var x = obj.method().another()\n"),
        "var x = obj.method().another()\n"
    );
}

#[test]
fn test_string_concatenation() {
    assert_eq!(
        format("print(\"Hello\" + \" \" + \"World\")\n"),
        "print(\"Hello\" + \" \" + \"World\")\n"
    );
}

#[test]
fn test_comparison_operators() {
    assert_eq!(format("if x!=OK:\n\tpass\n"), "if x != OK:\n\tpass\n");
    assert_eq!(format("if x==1:\n\tpass\n"), "if x == 1:\n\tpass\n");
}

#[test]
fn test_boolean_operators() {
    assert!(format_ok("if x && y:\n\tpass\n"));
    assert!(format_ok("if x || y:\n\tpass\n"));
    assert!(format_ok("if !x:\n\tpass\n"));
}

#[test]
fn test_elif_body() {
    // elif clause should preserve its body content
    let input = "if x == 1:\n\tx = 2\nelif x == 3:\n\tx = 4\n";
    assert_eq!(format(input), input);
}

#[test]
fn test_complex_elif_chain() {
    let input = r#"if name == "main":
	resource = "ModelMesh"
elif !name.begins_with("ModelMesh_"):
	resource = "ModelMesh_" + str(name)
else:
	resource = name
"#;
    assert_eq!(format(input), input);
}

#[test]
fn test_multiline_dictionary() {
    let input = r#"var data := {
	"key1": "value1",
	"key2": "value2",
}
"#;
    assert_eq!(format(input), input);
}

#[test]
fn test_typed_default_parameter_multiple() {
    // Function with multiple typed parameters, some with defaults
    let input = "func save_file(path: String, with_model: bool = false, count: int = 10):\n\tpass\n";
    assert_eq!(format(input), input);
}

// =============================================================================
// AST Equivalence Tests
// =============================================================================
// These tests verify that formatting does not change the AST structure.
// This is the key safety invariant for the formatter.

#[test]
fn test_ast_equivalence_basic() {
    // Basic statements
    assert_ast_equivalent("extends Node2D\n");
    assert_ast_equivalent("class_name MyClass\n");
    assert_ast_equivalent("var x = 1\n");
    assert_ast_equivalent("var x: int = 1\n");
    assert_ast_equivalent("var x := 1\n");
    assert_ast_equivalent("const MAX = 100\n");
    assert_ast_equivalent("signal my_signal\n");
    assert_ast_equivalent("signal data_received(data, sender)\n");
}

#[test]
fn test_ast_equivalence_functions() {
    assert_ast_equivalent("func foo():\n\tpass\n");
    assert_ast_equivalent("func foo(a: int, b: String) -> void:\n\treturn\n");
    assert_ast_equivalent("func foo(x = 5, y: int = 10):\n\tpass\n");
}

#[test]
fn test_static_function() {
    let input = "static func bar():\n\tpass\n";
    let output = format(input);
    assert!(output.starts_with("static func"), "static keyword should be preserved, got: {}", output);
    assert_ast_equivalent(input);
}

#[test]
fn test_annotations() {
    // @export annotation should be preserved
    let input = "@export var speed: float = 10.0\n";
    let output = format(input);
    assert!(output.contains("@export"), "@export should be preserved, got: {}", output);
    assert_ast_equivalent(input);
}

#[test]
fn test_match_statement() {
    let input = "match x:\n\t1:\n\t\tpass\n\t_:\n\t\tpass\n";
    let output = format(input);
    assert!(output.contains("match x:"), "match statement should be preserved, got: {}", output);
    assert_ast_equivalent(input);
}

#[test]
fn test_ast_equivalence_control_flow() {
    assert_ast_equivalent("if x:\n\tpass\n");
    assert_ast_equivalent("if x:\n\tpass\nelse:\n\tpass\n");
    assert_ast_equivalent("if x:\n\tpass\nelif y:\n\tpass\nelse:\n\tpass\n");
    assert_ast_equivalent("for i in range(10):\n\tpass\n");
    assert_ast_equivalent("while true:\n\tbreak\n");
    assert_ast_equivalent("match x:\n\t1:\n\t\tpass\n\t_:\n\t\tpass\n");
}

#[test]
fn test_ast_equivalence_expressions() {
    assert_ast_equivalent("var x = 1 + 2\n");
    assert_ast_equivalent("var x = a && b\n");
    assert_ast_equivalent("var x = a < b\n");
    assert_ast_equivalent("var x = -a\n");
    assert_ast_equivalent("var x = !a\n");
    assert_ast_equivalent("var x = foo()\n");
    assert_ast_equivalent("var x = obj.method()\n");
    assert_ast_equivalent("var x = arr[0]\n");
    assert_ast_equivalent("var x = [1, 2, 3]\n");
    assert_ast_equivalent("var x = {a: 1, b: 2}\n");
}

#[test]
fn test_ast_equivalence_whitespace_changes() {
    // These have different whitespace but should produce same AST
    assert_ast_equivalent("var x=1\n");
    assert_ast_equivalent("var   x   =   1\n");
    assert_ast_equivalent("func foo(a:int,b:String):\n\tpass\n");
    assert_ast_equivalent("if x==1:\n\tpass\n");
}

#[test]
fn test_ast_equivalence_multiline_dict() {
    // Multiline dictionary should have same AST as single-line
    let multiline = r#"var d = {
	a: 1,
	b: 2,
}
"#;
    assert_ast_equivalent(multiline);
}

#[test]
fn test_ast_equivalence_fixture() {
    // Format the test fixture file and verify AST equivalence
    let source = include_str!("fixtures/format/test_basic.gd");
    assert_ast_equivalent(source);
}

#[test]
fn test_idempotent_fixture() {
    // Formatting twice should give the same result as formatting once
    let source = include_str!("fixtures/format/test_basic.gd");
    let formatted_once = format(source);
    let formatted_twice = format(&formatted_once);
    assert_eq!(formatted_once, formatted_twice, "Formatting is not idempotent");
}

// =============================================================================
// Blank Line Preservation Tests (GDScript Style Guide Compliance)
// =============================================================================
// Per the official style guide:
// - "Use one blank line inside functions to separate logical sections"
// - 2 blank lines between function/class definitions at top level

#[test]
fn test_blank_lines_within_function_preserved() {
    // Blank lines to separate logical sections should be kept
    let input = "func foo():\n\tvar x = 1\n\tvar y = 2\n\n\tx = x + 1\n\ty = y + 1\n";
    assert_eq!(format(input), input);
}

#[test]
fn test_blank_lines_between_toplevel_vars_preserved() {
    let input = "extends Node\n\nvar gltf := GLTFDocument.new()\nvar gltf_state := GLTFState.new()\n\nvar key_remap := {}\n";
    assert_eq!(format(input), input);
}

#[test]
fn test_multiple_blank_lines_collapsed_to_max() {
    // More than 2 blank lines at top level should be collapsed to 2
    let input = "extends Node\n\n\n\n\nvar x = 1\n";
    let expected = "extends Node\n\n\nvar x = 1\n";
    assert_eq!(format(input), expected);

    // More than 1 blank line within function should be collapsed to 1
    let input2 = "func foo():\n\tvar x = 1\n\n\n\n\tvar y = 2\n";
    let expected2 = "func foo():\n\tvar x = 1\n\n\tvar y = 2\n";
    assert_eq!(format(input2), expected2);
}

#[test]
fn test_inline_comment_two_spaces() {
    // Official guide says 2 spaces before inline comment
    let input = "var x = 1  # comment\n";
    assert_eq!(format(input), input);
    // Single space should be corrected to two spaces
    assert_eq!(format("var x = 1 # comment\n"), "var x = 1  # comment\n");
}
