pub mod ast_check;
mod comments;
mod context;
mod nodes;
mod options;
mod output;
pub mod reorder;
mod skip_regions;

pub use ast_check::{compare_ast_with_source, AstCheckResult};
pub use context::FormatContext;
pub use options::{FormatOptions, IndentStyle};
pub use output::{FormattedLine, FormattedOutput};
pub use reorder::reorder_source;

use crate::parser;
use comments::Comments;
use skip_regions::SkipRegions;

/// Format GDScript source code according to the official style guide.
/// Note: This does NOT reorder - call `reorder_source` separately if needed.
pub fn run_formatter(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    // Parse the source
    let tree = parser::parse(source).map_err(FormatError::Parse)?;

    // Extract comments (not in AST)
    let comments = Comments::extract(source);

    // Find skip regions (# fmt: off/on)
    let skip_regions = SkipRegions::parse(source);

    // Create formatting context
    let mut ctx = FormatContext::new(source, &tree, options, skip_regions);

    // Format the tree
    let root = tree.root_node();
    nodes::format_node(root, &mut ctx);

    // Inject comments back
    ctx.output.inject_comments(&comments, source);

    // Build final output
    Ok(ctx.output.to_string(options))
}

#[derive(Debug)]
pub enum FormatError {
    Parse(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for FormatError {}
