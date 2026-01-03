use std::collections::HashMap;

/// Extracted comments from source code.
///
/// Comments are not part of the tree-sitter AST, so we extract them
/// separately and inject them back after formatting.
#[derive(Debug, Default)]
pub struct Comments {
    /// Standalone comments (entire line is a comment), keyed by line number (1-indexed).
    standalone: HashMap<usize, String>,
    /// Inline comments (code followed by comment), keyed by line number (1-indexed).
    inline: HashMap<usize, String>,
}

impl Comments {
    /// Extract comments from source code.
    pub fn extract(source: &str) -> Self {
        let mut standalone = HashMap::new();
        let mut inline = HashMap::new();

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1; // 1-indexed
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Check if line starts with # (standalone comment)
            if trimmed.starts_with('#') {
                // Preserve original indentation for standalone comments
                standalone.insert(line_num, line.to_string());
            } else if let Some(hash_pos) = find_comment_start(line) {
                // Line has code followed by comment
                let comment = line[hash_pos..].to_string();
                inline.insert(line_num, comment);
            }
        }

        Self { standalone, inline }
    }

    /// Get a standalone comment for a line.
    pub fn get_standalone(&self, line: usize) -> Option<&String> {
        self.standalone.get(&line)
    }

    /// Get an inline comment for a line.
    pub fn get_inline(&self, line: usize) -> Option<&String> {
        self.inline.get(&line)
    }
}

/// Find the start of a comment in a line, handling strings.
fn find_comment_start(line: &str) -> Option<usize> {
    let mut in_string = false;
    let mut string_char = ' ';
    let mut prev_char = ' ';
    let chars: Vec<char> = line.chars().collect();

    for (i, &ch) in chars.iter().enumerate() {
        if in_string {
            // Check for end of string (not escaped)
            if ch == string_char && prev_char != '\\' {
                in_string = false;
            }
        } else {
            // Check for start of string
            if ch == '"' || ch == '\'' {
                in_string = true;
                string_char = ch;
            } else if ch == '#' {
                return Some(i);
            }
        }
        prev_char = ch;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standalone_comment() {
        let source = "# This is a comment\nvar x = 1";
        let comments = Comments::extract(source);
        assert!(comments.get_standalone(1).is_some());
        assert!(comments.get_standalone(2).is_none());
        assert_eq!(
            comments.get_standalone(1),
            Some(&"# This is a comment".to_string())
        );
    }

    #[test]
    fn test_inline_comment() {
        let source = "var x = 1  # inline comment";
        let comments = Comments::extract(source);
        assert!(comments.get_standalone(1).is_none());
        assert!(comments.get_inline(1).is_some());
        assert_eq!(
            comments.get_inline(1),
            Some(&"# inline comment".to_string())
        );
    }

    #[test]
    fn test_comment_in_string() {
        let source = "var x = \"# not a comment\"";
        let comments = Comments::extract(source);
        assert!(comments.get_standalone(1).is_none());
        assert!(comments.get_inline(1).is_none());
    }

    #[test]
    fn test_indented_standalone_comment() {
        let source = "func foo():\n\t# indented comment\n\tpass";
        let comments = Comments::extract(source);
        assert!(comments.get_standalone(2).is_some());
        assert_eq!(
            comments.get_standalone(2),
            Some(&"\t# indented comment".to_string())
        );
    }
}
