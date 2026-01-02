use serde::{Deserialize, Serialize};
use tree_sitter::Node;

use crate::config::RuleConfig;
use crate::lint::LintContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    #[default]
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Naming,
    Format,
    Basic,
    Design,
    Style,
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleCategory::Naming => write!(f, "naming"),
            RuleCategory::Format => write!(f, "format"),
            RuleCategory::Basic => write!(f, "basic"),
            RuleCategory::Design => write!(f, "design"),
            RuleCategory::Style => write!(f, "style"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuleMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub category: RuleCategory,
    pub default_severity: Severity,
    pub description: &'static str,
}

pub trait Rule: Send + Sync {
    fn meta(&self) -> &RuleMetadata;

    fn interested_node_kinds(&self) -> Option<&'static [&'static str]> {
        None
    }

    fn check_node(&self, node: Node<'_>, ctx: &mut LintContext<'_>);

    fn check_file_start(&self, _ctx: &mut LintContext<'_>) {}

    fn check_file_end(&self, _ctx: &mut LintContext<'_>) {}

    fn configure(&mut self, _config: &RuleConfig) -> Result<(), String> {
        Ok(())
    }
}
