use tree_sitter::Tree;

use super::options::FormatOptions;
use super::output::FormattedOutput;
use super::skip_regions::SkipRegions;

/// Mutable context passed during formatting.
pub struct FormatContext<'a> {
    /// Original source code.
    pub source: &'a str,
    /// Source lines for extracting original text.
    pub lines: Vec<&'a str>,
    /// Parsed tree-sitter tree.
    pub tree: &'a Tree,
    /// Formatting options.
    pub options: &'a FormatOptions,
    /// Current indentation level.
    pub indent_level: usize,
    /// Regions to skip formatting (# fmt: off/on).
    pub skip_regions: SkipRegions,
    /// Output being built.
    pub output: FormattedOutput,
}

impl<'a> FormatContext<'a> {
    /// Create a new formatting context.
    pub fn new(
        source: &'a str,
        tree: &'a Tree,
        options: &'a FormatOptions,
        skip_regions: SkipRegions,
    ) -> Self {
        Self {
            source,
            lines: source.lines().collect(),
            tree,
            options,
            indent_level: 0,
            skip_regions,
            output: FormattedOutput::new(),
        }
    }

    /// Get the current indentation string.
    pub fn indent_str(&self) -> String {
        self.options.indent_style.as_str().repeat(self.indent_level)
    }

    /// Increase indentation level.
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level.
    pub fn dedent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }

    /// Check if a line number is in a skip region.
    pub fn is_skipped(&self, line: usize) -> bool {
        self.skip_regions.is_skipped(line)
    }

    /// Get a line from the original source (1-indexed).
    pub fn get_source_line(&self, line: usize) -> Option<&'a str> {
        if line == 0 || line > self.lines.len() {
            None
        } else {
            Some(self.lines[line - 1])
        }
    }

    /// Calculate the visual width of a string (tabs count as indent width).
    pub fn visual_width(&self, s: &str) -> usize {
        let tab_width = self.options.indent_style.width();
        s.chars()
            .map(|c| if c == '\t' { tab_width } else { 1 })
            .sum()
    }

    /// Check if a string would exceed max line length.
    pub fn exceeds_line_length(&self, s: &str) -> bool {
        self.visual_width(s) > self.options.max_line_length
    }
}
