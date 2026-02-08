//! Yamllint-compatible configuration parser

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Yamllint configuration file
#[derive(Debug, Clone, Deserialize)]
pub struct YamllintConfig {
    /// Extends another configuration (e.g., "default", "relaxed")
    #[serde(default)]
    pub extends: Option<String>,

    /// YAML-specific options
    #[serde(default)]
    pub yaml_files: Option<Vec<String>>,

    /// Files to ignore
    #[serde(default)]
    pub ignore: Option<Vec<String>>,

    /// Rule configurations
    #[serde(default)]
    pub rules: HashMap<String, YamllintRuleConfig>,
}

/// Configuration for a single yamllint rule
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum YamllintRuleConfig {
    /// Simple enable/disable
    Enabled(bool),
    /// Enable with string
    EnableStr(String),
    /// Detailed configuration
    Detailed(HashMap<String, serde_yaml_ng::Value>),
}

impl YamllintRuleConfig {
    pub fn is_enabled(&self) -> bool {
        match self {
            YamllintRuleConfig::Enabled(b) => *b,
            YamllintRuleConfig::EnableStr(s) => s != "disable",
            YamllintRuleConfig::Detailed(map) => !map
                .get("disable")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }

    pub fn get_option<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        match self {
            YamllintRuleConfig::Detailed(map) => map
                .get(key)
                .and_then(|v| serde_yaml_ng::from_value(v.clone()).ok()),
            _ => None,
        }
    }

    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.get_option(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_option(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get_option(key)
    }

    pub fn get_list(&self, key: &str) -> Option<Vec<String>> {
        self.get_option(key)
    }
}

impl Default for YamllintConfig {
    fn default() -> Self {
        Self {
            extends: Some("default".to_string()),
            yaml_files: None,
            ignore: None,
            rules: HashMap::new(),
        }
    }
}

impl YamllintConfig {
    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read yamllint config: {}", path.display()))?;

        let config: YamllintConfig = serde_yaml_ng::from_str(&content)
            .with_context(|| format!("Failed to parse yamllint config: {}", path.display()))?;

        Ok(config)
    }

    /// Find yamllint config file in directory hierarchy
    pub fn find_config(start_dir: &Path) -> Option<PathBuf> {
        let config_names = [".yamllint", ".yamllint.yaml", ".yamllint.yml"];

        let mut current = Some(start_dir);
        while let Some(dir) = current {
            for name in &config_names {
                let path = dir.join(name);
                if path.exists() {
                    return Some(path);
                }
            }
            current = dir.parent();
        }

        // Check XDG config home
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config).join("yamllint/config");
            if path.exists() {
                return Some(path);
            }
        }

        // Check ~/.config/yamllint/config
        if let Some(home) = dirs_home() {
            let path = home.join(".config/yamllint/config");
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Get the default configuration
    pub fn default_config() -> Self {
        let mut rules = HashMap::new();

        // Default yamllint rules
        rules.insert("braces".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert("brackets".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert("colons".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert("commas".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert("comments".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert(
            "comments-indentation".to_string(),
            YamllintRuleConfig::Enabled(true),
        );
        rules.insert(
            "document-end".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        rules.insert(
            "document-start".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        rules.insert("empty-lines".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert(
            "empty-values".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        rules.insert("hyphens".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert("indentation".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert(
            "key-duplicates".to_string(),
            YamllintRuleConfig::Enabled(true),
        );
        rules.insert(
            "key-ordering".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        rules.insert("line-length".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert(
            "new-line-at-end-of-file".to_string(),
            YamllintRuleConfig::Enabled(true),
        );
        rules.insert("new-lines".to_string(), YamllintRuleConfig::Enabled(true));
        rules.insert(
            "trailing-spaces".to_string(),
            YamllintRuleConfig::Enabled(true),
        );
        rules.insert("truthy".to_string(), YamllintRuleConfig::Enabled(true));

        Self {
            extends: None,
            yaml_files: None,
            ignore: None,
            rules,
        }
    }

    /// Get the relaxed configuration
    pub fn relaxed_config() -> Self {
        let mut config = Self::default_config();

        // Relaxed mode disables some stricter rules
        config.rules.insert(
            "braces".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert(
                    "min-spaces-inside".to_string(),
                    serde_yaml_ng::Value::Number(0.into()),
                );
                m.insert(
                    "max-spaces-inside".to_string(),
                    serde_yaml_ng::Value::Number(1.into()),
                );
                m
            }),
        );
        config.rules.insert(
            "brackets".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert(
                    "min-spaces-inside".to_string(),
                    serde_yaml_ng::Value::Number(0.into()),
                );
                m.insert(
                    "max-spaces-inside".to_string(),
                    serde_yaml_ng::Value::Number(1.into()),
                );
                m
            }),
        );
        config.rules.insert(
            "colons".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert(
                    "max-spaces-after".to_string(),
                    serde_yaml_ng::Value::Number((-1).into()),
                );
                m
            }),
        );
        config.rules.insert(
            "commas".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert(
                    "max-spaces-after".to_string(),
                    serde_yaml_ng::Value::Number((-1).into()),
                );
                m
            }),
        );
        config
            .rules
            .insert("comments".to_string(), YamllintRuleConfig::Enabled(false));
        config.rules.insert(
            "comments-indentation".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        config.rules.insert(
            "empty-lines".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert("max".to_string(), serde_yaml_ng::Value::Number(3.into()));
                m
            }),
        );
        config
            .rules
            .insert("hyphens".to_string(), YamllintRuleConfig::Enabled(false));
        config.rules.insert(
            "indentation".to_string(),
            YamllintRuleConfig::Enabled(false),
        );
        config.rules.insert(
            "line-length".to_string(),
            YamllintRuleConfig::Detailed({
                let mut m = HashMap::new();
                m.insert("max".to_string(), serde_yaml_ng::Value::Number(120.into()));
                m.insert(
                    "allow-non-breakable-inline-mappings".to_string(),
                    serde_yaml_ng::Value::Bool(true),
                );
                m
            }),
        );
        config
            .rules
            .insert("truthy".to_string(), YamllintRuleConfig::Enabled(false));

        config
    }

    /// Merge with a base configuration
    pub fn merge_with_base(&mut self, base: YamllintConfig) {
        for (rule, config) in base.rules {
            self.rules.entry(rule).or_insert(config);
        }

        if self.yaml_files.is_none() {
            self.yaml_files = base.yaml_files;
        }
        if self.ignore.is_none() {
            self.ignore = base.ignore;
        }
    }

    /// Resolve extends and return final configuration
    pub fn resolve(mut self) -> Self {
        if let Some(extends) = &self.extends {
            let base = match extends.as_str() {
                "default" => Self::default_config(),
                "relaxed" => Self::relaxed_config(),
                _ => Self::default_config(), // Unknown, use default
            };
            self.merge_with_base(base);
        }
        self.extends = None;
        self
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, rule: &str) -> bool {
        self.rules
            .get(rule)
            .map(|c| c.is_enabled())
            .unwrap_or(false)
    }

    /// Get rule configuration
    pub fn rule_config(&self, rule: &str) -> Option<&YamllintRuleConfig> {
        self.rules.get(rule)
    }
}

/// Get user's home directory
fn dirs_home() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let yaml = r#"
extends: default
rules:
  line-length:
    max: 100
  trailing-spaces: enable
  document-start: disable
"#;

        let config: YamllintConfig = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(config.extends, Some("default".to_string()));
        assert!(config.rules.contains_key("line-length"));
    }

    #[test]
    fn test_default_config() {
        let config = YamllintConfig::default_config();
        assert!(config.is_rule_enabled("trailing-spaces"));
        assert!(config.is_rule_enabled("line-length"));
        assert!(!config.is_rule_enabled("document-start"));
    }
}
