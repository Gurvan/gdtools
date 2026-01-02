use std::path::PathBuf;

use gdlint::config::Config;
use gdlint::lint::run_linter;
use gdlint::rules::all_rules;

fn lint_code(source: &str) -> Vec<(String, String)> {
    let config = Config::default();
    let rules = all_rules();
    let path = PathBuf::from("test.gd");

    let diagnostics = run_linter(source, &path, &rules, &config).unwrap();
    diagnostics
        .into_iter()
        .map(|d| (d.rule_id, d.message))
        .collect()
}

fn has_rule_violation(source: &str, rule_id: &str) -> bool {
    lint_code(source).iter().any(|(id, _)| id == rule_id)
}

// ============================================================================
// Naming Rules Tests
// ============================================================================

#[test]
fn test_function_name_snake_case() {
    assert!(!has_rule_violation("func my_function():\n    pass", "function-name"));
    assert!(has_rule_violation("func MyFunction():\n    pass", "function-name"));
    assert!(has_rule_violation("func myFunction():\n    pass", "function-name"));
}

#[test]
fn test_function_name_private() {
    assert!(!has_rule_violation("func _private_func():\n    pass", "function-name"));
    assert!(has_rule_violation("func _PrivateFunc():\n    pass", "function-name"));
}

#[test]
fn test_function_name_signal_handler() {
    // Signal handlers like _on_Button_pressed should be valid
    assert!(!has_rule_violation("func _on_Button_pressed():\n    pass", "function-name"));
    assert!(!has_rule_violation("func _on_Timer_timeout():\n    pass", "function-name"));
}

#[test]
fn test_class_name_pascal_case() {
    assert!(!has_rule_violation("class MyClass:\n    pass", "class-name"));
    assert!(has_rule_violation("class my_class:\n    pass", "class-name"));
    assert!(has_rule_violation("class myClass:\n    pass", "class-name"));
}

#[test]
fn test_signal_name_snake_case() {
    assert!(!has_rule_violation("signal my_signal", "signal-name"));
    assert!(!has_rule_violation("signal health_changed", "signal-name"));
    assert!(has_rule_violation("signal MySignal", "signal-name"));
    assert!(has_rule_violation("signal healthChanged", "signal-name"));
}

#[test]
fn test_constant_name_upper_snake() {
    assert!(!has_rule_violation("const MAX_VALUE = 100", "constant-name"));
    assert!(!has_rule_violation("const SPEED = 5.0", "constant-name"));
    assert!(has_rule_violation("const maxValue = 100", "constant-name"));
    assert!(has_rule_violation("const max_value = 100", "constant-name"));
}

#[test]
fn test_variable_name_snake_case() {
    assert!(!has_rule_violation("var my_var = 0", "variable-name"));
    assert!(!has_rule_violation("var _private_var = 0", "variable-name"));
    assert!(has_rule_violation("var MyVar = 0", "variable-name"));
    assert!(has_rule_violation("var myVar = 0", "variable-name"));
}

#[test]
fn test_enum_name_pascal_case() {
    assert!(!has_rule_violation("enum State { IDLE, RUNNING }", "enum-name"));
    assert!(!has_rule_violation("enum MyEnum { A, B }", "enum-name"));
    // MYENUM is valid PascalCase (starts with uppercase, followed by alphanumeric)
    assert!(!has_rule_violation("enum MYENUM { A, B }", "enum-name"));
    assert!(has_rule_violation("enum my_enum { A, B }", "enum-name"));
    assert!(has_rule_violation("enum myEnum { A, B }", "enum-name"));
}

#[test]
fn test_enum_element_name_upper_snake() {
    assert!(!has_rule_violation("enum State { IDLE, RUNNING }", "enum-element-name"));
    assert!(!has_rule_violation("enum E { MAX_VALUE }", "enum-element-name"));
    assert!(has_rule_violation("enum State { idle, running }", "enum-element-name"));
    assert!(has_rule_violation("enum State { Idle, Running }", "enum-element-name"));
}

// ============================================================================
// Format Rules Tests
// ============================================================================

#[test]
fn test_max_line_length() {
    let short_line = "var x = 1";
    assert!(!has_rule_violation(short_line, "max-line-length"));

    // Default is 100 chars
    let long_line = format!("var x = \"{}\"", "a".repeat(100));
    assert!(has_rule_violation(&long_line, "max-line-length"));
}

#[test]
fn test_trailing_whitespace() {
    assert!(!has_rule_violation("var x = 1", "trailing-whitespace"));
    assert!(has_rule_violation("var x = 1 ", "trailing-whitespace"));
    assert!(has_rule_violation("var x = 1\t", "trailing-whitespace"));
}

#[test]
fn test_mixed_tabs_spaces() {
    assert!(!has_rule_violation("    var x = 1", "mixed-tabs-spaces"));
    assert!(!has_rule_violation("\tvar x = 1", "mixed-tabs-spaces"));
    assert!(has_rule_violation("\t var x = 1", "mixed-tabs-spaces"));
    assert!(has_rule_violation(" \tvar x = 1", "mixed-tabs-spaces"));
}

#[test]
fn test_max_file_lines() {
    let short_file = "var x = 1\nvar y = 2";
    assert!(!has_rule_violation(short_file, "max-file-lines"));

    // Default is 1000 lines
    let long_file = (0..1001).map(|i| format!("var v{} = {}", i, i)).collect::<Vec<_>>().join("\n");
    assert!(has_rule_violation(&long_file, "max-file-lines"));
}

// ============================================================================
// Basic Rules Tests
// ============================================================================

#[test]
fn test_unnecessary_pass() {
    // Pass alone is fine
    assert!(!has_rule_violation("func f():\n    pass", "unnecessary-pass"));

    // Pass with other statements is unnecessary
    assert!(has_rule_violation("func f():\n    var x = 1\n    pass", "unnecessary-pass"));
}

#[test]
fn test_unused_argument() {
    // Used argument
    assert!(!has_rule_violation("func f(x):\n    print(x)", "unused-argument"));

    // Unused argument
    assert!(has_rule_violation("func f(x):\n    pass", "unused-argument"));

    // Underscore prefix is ignored
    assert!(!has_rule_violation("func f(_x):\n    pass", "unused-argument"));
}

#[test]
fn test_unused_argument_typed() {
    // Typed used argument
    assert!(!has_rule_violation("func f(x: int):\n    print(x)", "unused-argument"));

    // Typed unused argument
    assert!(has_rule_violation("func f(x: int):\n    pass", "unused-argument"));
}

#[test]
fn test_comparison_with_itself() {
    assert!(!has_rule_violation("if x == y:\n    pass", "comparison-with-itself"));
    assert!(has_rule_violation("if x == x:\n    pass", "comparison-with-itself"));
    assert!(has_rule_violation("if foo == foo:\n    pass", "comparison-with-itself"));
}

#[test]
fn test_duplicated_load() {
    let no_dup = r#"
var a = load("res://a.tscn")
var b = load("res://b.tscn")
"#;
    assert!(!has_rule_violation(no_dup, "duplicated-load"));

    let dup = r#"
var a = load("res://a.tscn")
var b = load("res://a.tscn")
"#;
    assert!(has_rule_violation(dup, "duplicated-load"));
}

#[test]
fn test_expression_not_assigned() {
    // Call expressions are OK (side effects)
    assert!(!has_rule_violation("func f():\n    print(1)", "expression-not-assigned"));

    // Assignments are OK
    assert!(!has_rule_violation("func f():\n    var x = 1 + 2", "expression-not-assigned"));

    // Standalone arithmetic is likely a bug
    assert!(has_rule_violation("func f():\n    1 + 2", "expression-not-assigned"));
}

// ============================================================================
// Design Rules Tests
// ============================================================================

#[test]
fn test_max_function_args() {
    let few_args = "func f(a, b, c):\n    pass";
    assert!(!has_rule_violation(few_args, "max-function-args"));

    // Default is 10
    let many_args = "func f(a, b, c, d, e, f, g, h, i, j, k):\n    pass";
    assert!(has_rule_violation(many_args, "max-function-args"));
}

#[test]
fn test_max_returns() {
    let few_returns = r#"
func f(x):
    if x > 0:
        return 1
    return 0
"#;
    assert!(!has_rule_violation(few_returns, "max-returns"));

    // Default is 6
    let many_returns = r#"
func f(x):
    if x == 1: return 1
    if x == 2: return 2
    if x == 3: return 3
    if x == 4: return 4
    if x == 5: return 5
    if x == 6: return 6
    return 0
"#;
    assert!(has_rule_violation(many_returns, "max-returns"));
}

#[test]
fn test_max_public_methods() {
    let few_methods = r#"
func a(): pass
func b(): pass
func c(): pass
"#;
    assert!(!has_rule_violation(few_methods, "max-public-methods"));
}

// ============================================================================
// Style Rules Tests
// ============================================================================

#[test]
fn test_no_elif_return() {
    // No elif after return - good
    let good = r#"
func f(x):
    if x > 0:
        return 1
    if x < 0:
        return -1
    return 0
"#;
    assert!(!has_rule_violation(good, "no-elif-return"));

    // elif after return - bad
    let bad = r#"
func f(x):
    if x > 0:
        return 1
    elif x < 0:
        return -1
    return 0
"#;
    assert!(has_rule_violation(bad, "no-elif-return"));
}

#[test]
fn test_no_else_return() {
    // No unnecessary else
    let good = r#"
func f(x):
    if x > 0:
        return 1
    return 0
"#;
    assert!(!has_rule_violation(good, "no-else-return"));

    // Unnecessary else after return
    let bad = r#"
func f(x):
    if x > 0:
        return 1
    else:
        return 0
"#;
    assert!(has_rule_violation(bad, "no-else-return"));
}

#[test]
fn test_class_definitions_order() {
    // Good order
    let good = r#"
extends Node
signal my_signal
const VALUE = 1
var my_var = 0
func my_func():
    pass
"#;
    assert!(!has_rule_violation(good, "class-definitions-order"));

    // Bad order - function before signal
    let bad = r#"
extends Node
func my_func():
    pass
signal my_signal
"#;
    assert!(has_rule_violation(bad, "class-definitions-order"));
}
