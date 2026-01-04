mod types;

pub use types::{Config, RuleConfig, RulesConfig};

use std::path::Path;

pub fn load_config(path: Option<&Path>) -> Result<Config, String> {
    if let Some(p) = path {
        let content =
            std::fs::read_to_string(p).map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    } else if let Some(found) = find_config_file() {
        let content = std::fs::read_to_string(&found)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
    } else {
        Ok(Config::default())
    }
}

fn find_config_file() -> Option<std::path::PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        let config_path = current.join("gdtools.toml");
        if config_path.exists() {
            return Some(config_path);
        }

        if !current.pop() {
            break;
        }
    }

    None
}
