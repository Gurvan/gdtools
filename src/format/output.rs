use super::comments::Comments;
use super::options::FormatOptions;

/// A single formatted line with optional source line mapping.
#[derive(Debug, Clone)]
pub struct FormattedLine {
    /// The source line number this came from (1-indexed), if known.
    pub source_line: Option<usize>,
    /// The formatted content (without trailing newline).
    pub content: String,
}

impl FormattedLine {
    /// Create a new formatted line.
    pub fn new(content: String) -> Self {
        Self {
            source_line: None,
            content,
        }
    }

    /// Create a formatted line with source mapping.
    pub fn with_source(content: String, source_line: usize) -> Self {
        Self {
            source_line: Some(source_line),
            content,
        }
    }

    /// Create an empty line.
    pub fn empty() -> Self {
        Self {
            source_line: None,
            content: String::new(),
        }
    }
}

/// Builder for formatted output.
#[derive(Debug, Default)]
pub struct FormattedOutput {
    lines: Vec<FormattedLine>,
}

impl FormattedOutput {
    /// Create a new empty output.
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    /// Add a formatted line.
    pub fn push(&mut self, line: FormattedLine) {
        self.lines.push(line);
    }

    /// Add a line with just content.
    pub fn push_line(&mut self, content: impl Into<String>) {
        self.lines.push(FormattedLine::new(content.into()));
    }

    /// Add a line with source mapping.
    pub fn push_mapped(&mut self, content: impl Into<String>, source_line: usize) {
        self.lines
            .push(FormattedLine::with_source(content.into(), source_line));
    }

    /// Add an empty line.
    pub fn push_empty(&mut self) {
        self.lines.push(FormattedLine::empty());
    }

    /// Add multiple empty lines, but ensure we don't exceed 2 consecutive.
    pub fn push_blank_lines(&mut self, count: usize) {
        let count = count.min(2); // Never more than 2 consecutive blank lines
        let trailing_blanks = self.trailing_blank_count();
        let to_add = count.saturating_sub(trailing_blanks);
        for _ in 0..to_add {
            self.push_empty();
        }
    }

    /// Count trailing blank lines.
    fn trailing_blank_count(&self) -> usize {
        self.lines
            .iter()
            .rev()
            .take_while(|l| l.content.is_empty())
            .count()
    }

    /// Get the number of lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Inject comments back into the output.
    pub fn inject_comments(&mut self, comments: &Comments, source: &str) {
        // Collect all source lines that were already output (for verbatim content)
        let mut already_output: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for line in &self.lines {
            if let Some(src_line) = line.source_line {
                already_output.insert(src_line);
            }
        }

        // Build a lookup for what source line each output line corresponds to
        // We'll inject comments by looking at the source line context
        let mut new_lines: Vec<FormattedLine> = Vec::with_capacity(self.lines.len());
        let mut last_source_line = 0;
        let source_lines: Vec<&str> = source.lines().collect();

        for line in self.lines.drain(..) {
            // Before processing this line, check if we need to inject comments
            // between the last source line and the current position
            if let Some(src_line) = line.source_line {
                // Inject any standalone comments that appear between last_source_line and src_line
                for comment_line in (last_source_line + 1)..src_line {
                    // Skip if this line was already output (part of verbatim content)
                    if already_output.contains(&comment_line) {
                        continue;
                    }
                    if let Some(comment) = comments.get_standalone(comment_line) {
                        new_lines.push(FormattedLine::with_source(comment.clone(), comment_line));
                    }
                }
                last_source_line = src_line;
            } else {
                // This is a blank line (no source mapping)
                // Before adding it, check if there are comments that should appear first
                // We need to find the next source-mapped line to know the range
                // But we can only look ahead, which is expensive. Instead, we'll inject
                // comments that immediately follow the last source line.

                // Find comments that appear right after last_source_line
                // These should go before any blank lines we're about to add
                let mut comment_line = last_source_line + 1;
                while comment_line <= source_lines.len() {
                    // Stop if this line is not a comment or blank
                    if let Some(src) = source_lines.get(comment_line - 1) {
                        let trimmed = src.trim();
                        if trimmed.is_empty() {
                            // Blank line - stop looking for more comments here
                            break;
                        }
                        if !trimmed.starts_with('#') {
                            // Non-comment, non-blank - stop
                            break;
                        }
                        // It's a comment - inject it if not already output
                        if !already_output.contains(&comment_line) {
                            if let Some(comment) = comments.get_standalone(comment_line) {
                                new_lines.push(FormattedLine::with_source(comment.clone(), comment_line));
                                already_output.insert(comment_line);
                            }
                        }
                        last_source_line = comment_line;
                    }
                    comment_line += 1;
                }
            }

            // Check for inline comment on this line
            let content = if let Some(src_line) = line.source_line {
                if let Some(comment) = comments.get_inline(src_line) {
                    if line.content.is_empty() {
                        comment.clone()
                    } else if line.content.ends_with(comment) {
                        // Comment already present (from verbatim output), don't duplicate
                        line.content
                    } else {
                        format!("{}  {}", line.content, comment)
                    }
                } else {
                    line.content
                }
            } else {
                line.content
            };

            new_lines.push(FormattedLine {
                source_line: line.source_line,
                content,
            });
        }

        self.lines = new_lines;
    }

    /// Convert to final string output.
    pub fn to_string(&self, options: &FormatOptions) -> String {
        let mut result: Vec<&str> = self.lines.iter().map(|l| l.content.as_str()).collect();

        // Remove trailing blank lines (we'll add one back if needed)
        while result.last().map(|s| s.is_empty()).unwrap_or(false) {
            result.pop();
        }

        let mut output = result.join("\n");

        // Add trailing newline if configured
        if options.trailing_newline && !output.is_empty() {
            output.push('\n');
        }

        output
    }
}
