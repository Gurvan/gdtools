use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use regex::Regex;

static IGNORE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#\s*gdlint:\s*ignore\s*=\s*([a-z0-9_,-]+)").unwrap()
});

static DISABLE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#\s*gdlint:\s*disable\s*=\s*([a-z0-9_,-]+)").unwrap()
});

static ENABLE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"#\s*gdlint:\s*enable\s*=\s*([a-z0-9_,-]+)").unwrap()
});

#[derive(Debug, Default)]
pub struct Suppressions {
    line_suppressions: HashMap<usize, HashSet<String>>,
    disabled_rules: HashMap<String, Vec<(usize, Option<usize>)>>,
}

impl Suppressions {
    pub fn parse(source: &str) -> Self {
        let mut suppressions = Self::default();
        let mut currently_disabled: HashMap<String, usize> = HashMap::new();

        for (line_idx, line) in source.lines().enumerate() {
            let line_num = line_idx + 1;

            if let Some(caps) = IGNORE_PATTERN.captures(line) {
                let rules = parse_rule_list(&caps[1]);
                for rule in rules {
                    suppressions
                        .line_suppressions
                        .entry(line_num)
                        .or_default()
                        .insert(rule.clone());
                    suppressions
                        .line_suppressions
                        .entry(line_num + 1)
                        .or_default()
                        .insert(rule);
                }
            }

            if let Some(caps) = DISABLE_PATTERN.captures(line) {
                let rules = parse_rule_list(&caps[1]);
                for rule in rules {
                    currently_disabled.insert(rule, line_num);
                }
            }

            if let Some(caps) = ENABLE_PATTERN.captures(line) {
                let rules = parse_rule_list(&caps[1]);
                for rule in rules {
                    if let Some(start_line) = currently_disabled.remove(&rule) {
                        suppressions
                            .disabled_rules
                            .entry(rule)
                            .or_default()
                            .push((start_line, Some(line_num)));
                    }
                }
            }
        }

        for (rule, start_line) in currently_disabled {
            suppressions
                .disabled_rules
                .entry(rule)
                .or_default()
                .push((start_line, None));
        }

        suppressions
    }

    pub fn is_suppressed(&self, rule_id: &str, line: usize) -> bool {
        if self
            .line_suppressions
            .get(&line)
            .map(|s| s.contains(rule_id))
            .unwrap_or(false)
        {
            return true;
        }

        if let Some(ranges) = self.disabled_rules.get(rule_id) {
            for (start, end) in ranges {
                let in_range = match end {
                    Some(end_line) => line >= *start && line <= *end_line,
                    None => line >= *start,
                };
                if in_range {
                    return true;
                }
            }
        }

        false
    }
}

fn parse_rule_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|r| r.trim().to_string())
        .filter(|r| !r.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_single_line() {
        let source = "# gdlint:ignore=function-name\nfunc BadName(): pass";
        let suppressions = Suppressions::parse(source);
        assert!(suppressions.is_suppressed("function-name", 1));
        assert!(suppressions.is_suppressed("function-name", 2));
        assert!(!suppressions.is_suppressed("function-name", 3));
    }

    #[test]
    fn test_disable_enable_range() {
        let source = r#"
# gdlint:disable=max-line-length
some long line here
another long line
# gdlint:enable=max-line-length
normal line
"#;
        let suppressions = Suppressions::parse(source);
        assert!(suppressions.is_suppressed("max-line-length", 2));
        assert!(suppressions.is_suppressed("max-line-length", 3));
        assert!(suppressions.is_suppressed("max-line-length", 4));
        assert!(suppressions.is_suppressed("max-line-length", 5));
        assert!(!suppressions.is_suppressed("max-line-length", 6));
    }
}
