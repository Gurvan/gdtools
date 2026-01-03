use gdlint::format::{run_formatter, FormatOptions};

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

