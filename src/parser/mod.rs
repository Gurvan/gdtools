use tree_sitter::{Language, Parser, Tree};

pub fn language() -> Language {
    tree_sitter_gdscript::LANGUAGE.into()
}

pub fn parse(source: &str) -> Result<Tree, String> {
    let mut parser = Parser::new();
    parser
        .set_language(&language())
        .map_err(|e| format!("Failed to set language: {}", e))?;
    parser
        .parse(source, None)
        .ok_or_else(|| "Failed to parse source".to_string())
}
