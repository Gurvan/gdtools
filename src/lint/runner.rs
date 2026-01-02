use std::path::Path;

use tree_sitter::TreeCursor;

use crate::config::Config;
use crate::lint::{Diagnostic, LintContext, Rule};
use crate::parser::parse;

pub fn run_linter(
    source: &str,
    file_path: &Path,
    rules: &[Box<dyn Rule>],
    config: &Config,
) -> Result<Vec<Diagnostic>, String> {
    let tree = parse(source)?;
    let mut ctx = LintContext::new(source, &tree, file_path, config);

    for rule in rules {
        rule.check_file_start(&mut ctx);
    }

    let interested_kinds = build_interest_map(rules);
    traverse_and_check(&tree.root_node(), &mut ctx, rules, &interested_kinds);

    for rule in rules {
        rule.check_file_end(&mut ctx);
    }

    Ok(ctx.into_diagnostics())
}

fn build_interest_map(rules: &[Box<dyn Rule>]) -> Vec<(usize, Option<&'static [&'static str]>)> {
    rules
        .iter()
        .enumerate()
        .map(|(i, r)| (i, r.interested_node_kinds()))
        .collect()
}

fn traverse_and_check(
    root: &tree_sitter::Node<'_>,
    ctx: &mut LintContext<'_>,
    rules: &[Box<dyn Rule>],
    interest_map: &[(usize, Option<&'static [&'static str]>)],
) {
    let mut cursor = root.walk();
    traverse_recursive(&mut cursor, ctx, rules, interest_map);
}

fn traverse_recursive(
    cursor: &mut TreeCursor<'_>,
    ctx: &mut LintContext<'_>,
    rules: &[Box<dyn Rule>],
    interest_map: &[(usize, Option<&'static [&'static str]>)],
) {
    let node = cursor.node();
    let kind = node.kind();

    for (idx, interests) in interest_map {
        let should_check = match interests {
            None => true,
            Some(kinds) => kinds.contains(&kind),
        };

        if should_check {
            rules[*idx].check_node(node, ctx);
        }
    }

    if cursor.goto_first_child() {
        loop {
            traverse_recursive(cursor, ctx, rules, interest_map);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}
