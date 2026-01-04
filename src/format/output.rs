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

        let source_lines: Vec<&str> = source.lines().collect();
        let mut new_lines: Vec<FormattedLine> = Vec::with_capacity(self.lines.len());
        let mut last_source_line = 0;

        // Collect all lines with their indices for look-ahead
        let lines_vec: Vec<FormattedLine> = self.lines.drain(..).collect();

        for (i, line) in lines_vec.iter().enumerate() {
            if let Some(src_line) = line.source_line {
                // This line has a source mapping
                // Inject any standalone comments that appear between last_source_line and src_line
                for comment_line in (last_source_line + 1)..src_line {
                    if already_output.contains(&comment_line) {
                        continue;
                    }
                    if let Some(comment) = comments.get_standalone(comment_line) {
                        new_lines.push(FormattedLine::with_source(comment.clone(), comment_line));
                        already_output.insert(comment_line);
                    }
                }
                last_source_line = src_line;

                // Add this line with inline comment if present
                let content = if let Some(comment) = comments.get_inline(src_line) {
                    if line.content.is_empty() {
                        comment.clone()
                    } else if line.content.ends_with(comment) {
                        line.content.clone()
                    } else {
                        format!("{}  {}", line.content, comment)
                    }
                } else {
                    line.content.clone()
                };

                new_lines.push(FormattedLine {
                    source_line: Some(src_line),
                    content,
                });
            } else {
                // This is a blank line (no source mapping)
                // Before adding the blank line, check if there are comments that should go before it
                // Look ahead to find the next source-mapped line
                let next_src_line = lines_vec[(i + 1)..].iter().find_map(|l| l.source_line);

                if let Some(next_sl) = next_src_line {
                    // Determine which comments belong BEFORE the blank lines (with previous code)
                    // vs AFTER the blank lines (with next code).
                    //
                    // Strategy: Find contiguous comment blocks and determine if each block
                    // belongs before or after the blank lines based on what follows it.
                    //
                    // A comment block belongs with PREVIOUS code if there's a blank line
                    // between it and the next non-comment content.

                    let mut comment_line = last_source_line + 1;
                    while comment_line < next_sl {
                        if already_output.contains(&comment_line) {
                            comment_line += 1;
                            continue;
                        }

                        // Check if this is a comment
                        let is_comment = source_lines
                            .get(comment_line - 1)
                            .map(|s| s.trim().starts_with('#'))
                            .unwrap_or(false);

                        if !is_comment {
                            comment_line += 1;
                            continue;
                        }

                        // Found a comment - find the end of this contiguous comment block
                        let block_start = comment_line;
                        let mut block_end = comment_line;
                        while block_end < next_sl {
                            let next_is_comment = source_lines
                                .get(block_end)
                                .map(|s| s.trim().starts_with('#'))
                                .unwrap_or(false);
                            if next_is_comment {
                                block_end += 1;
                            } else {
                                break;
                            }
                        }

                        // Check what follows this comment block
                        let line_after_block = block_end + 1;
                        let next_non_blank = (line_after_block..=next_sl).find(|&ln| {
                            source_lines
                                .get(ln - 1)
                                .map(|s| !s.trim().is_empty())
                                .unwrap_or(false)
                        });

                        // If there's a blank between the comment block and next content,
                        // or if there's nothing after (comment at end of gap), inject before blanks
                        let followed_by_blank = next_non_blank
                            .map(|nln| nln > line_after_block)
                            .unwrap_or(true);

                        if followed_by_blank {
                            // Inject the entire comment block before blank lines
                            for cl in block_start..=block_end {
                                if already_output.contains(&cl) {
                                    continue;
                                }
                                if let Some(comment) = comments.get_standalone(cl) {
                                    new_lines.push(FormattedLine::with_source(comment.clone(), cl));
                                    already_output.insert(cl);
                                    last_source_line = cl;
                                }
                            }
                        }

                        comment_line = block_end + 1;
                    }
                }

                // Now add the blank line
                new_lines.push(line.clone());
            }
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
