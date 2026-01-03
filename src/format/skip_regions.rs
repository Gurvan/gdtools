use once_cell::sync::Lazy;
use regex::Regex;

static FMT_OFF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\s*fmt:\s*off").unwrap());
static FMT_ON_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\s*fmt:\s*on").unwrap());

/// Tracks regions that should skip formatting (# fmt: off/on).
#[derive(Debug, Default)]
pub struct SkipRegions {
    /// Ranges of lines to skip (start, end) - both inclusive, 1-indexed.
    ranges: Vec<(usize, usize)>,
}

impl SkipRegions {
    /// Parse skip regions from source code.
    pub fn parse(source: &str) -> Self {
        let mut ranges = Vec::new();
        let mut current_start: Option<usize> = None;

        for (idx, line) in source.lines().enumerate() {
            let line_num = idx + 1; // 1-indexed

            if FMT_OFF_REGEX.is_match(line) {
                if current_start.is_none() {
                    current_start = Some(line_num);
                }
            } else if FMT_ON_REGEX.is_match(line) {
                if let Some(start) = current_start {
                    ranges.push((start, line_num));
                    current_start = None;
                }
            }
        }

        // If we have an unclosed # fmt: off, extend to end of file
        if let Some(start) = current_start {
            let line_count = source.lines().count();
            ranges.push((start, line_count));
        }

        Self { ranges }
    }

    /// Check if a line (1-indexed) is in a skip region.
    pub fn is_skipped(&self, line: usize) -> bool {
        self.ranges.iter().any(|(start, end)| line >= *start && line <= *end)
    }

    /// Get the skip region containing a line, if any.
    pub fn get_region(&self, line: usize) -> Option<(usize, usize)> {
        self.ranges
            .iter()
            .find(|(start, end)| line >= *start && line <= *end)
            .copied()
    }

    /// Check if empty (no skip regions).
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_skip_regions() {
        let source = "var x = 1\nvar y = 2";
        let regions = SkipRegions::parse(source);
        assert!(regions.is_empty());
        assert!(!regions.is_skipped(1));
        assert!(!regions.is_skipped(2));
    }

    #[test]
    fn test_single_skip_region() {
        let source = "var x = 1\n# fmt: off\nvar y   =   2\n# fmt: on\nvar z = 3";
        let regions = SkipRegions::parse(source);
        assert!(!regions.is_skipped(1)); // var x = 1
        assert!(regions.is_skipped(2)); // # fmt: off
        assert!(regions.is_skipped(3)); // var y = 2
        assert!(regions.is_skipped(4)); // # fmt: on
        assert!(!regions.is_skipped(5)); // var z = 3
    }

    #[test]
    fn test_unclosed_skip_region() {
        let source = "var x = 1\n# fmt: off\nvar y = 2\nvar z = 3";
        let regions = SkipRegions::parse(source);
        assert!(!regions.is_skipped(1));
        assert!(regions.is_skipped(2));
        assert!(regions.is_skipped(3));
        assert!(regions.is_skipped(4));
    }

    #[test]
    fn test_multiple_skip_regions() {
        let source = "# fmt: off\na\n# fmt: on\nb\n# fmt: off\nc\n# fmt: on";
        let regions = SkipRegions::parse(source);
        assert!(regions.is_skipped(1)); // # fmt: off
        assert!(regions.is_skipped(2)); // a
        assert!(regions.is_skipped(3)); // # fmt: on
        assert!(!regions.is_skipped(4)); // b
        assert!(regions.is_skipped(5)); // # fmt: off
        assert!(regions.is_skipped(6)); // c
        assert!(regions.is_skipped(7)); // # fmt: on
    }
}
