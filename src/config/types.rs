use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::lint::Severity;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub exclude: Vec<String>,
    pub rules: RulesConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RulesConfig {
    pub disable: Vec<String>,
    #[serde(flatten)]
    pub options: HashMap<String, RuleConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct RuleConfig {
    pub severity: Option<Severity>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(flatten)]
    pub options: HashMap<String, toml::Value>,
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        if self.rules.disable.contains(&rule_id.to_string()) {
            return false;
        }
        self.rules
            .options
            .get(rule_id)
            .map(|c| c.enabled)
            .unwrap_or(true)
    }

    pub fn get_rule_severity(&self, rule_id: &str, default: Severity) -> Severity {
        self.rules
            .options
            .get(rule_id)
            .and_then(|c| c.severity)
            .unwrap_or(default)
    }

    pub fn get_rule_config(&self, rule_id: &str) -> Option<&RuleConfig> {
        self.rules.options.get(rule_id)
    }
}
