//! Global configuration for the linter

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Global linter configuration
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Root directory for the lint operation
    pub root: PathBuf,
    /// File patterns to include
    pub include: Vec<String>,
    /// File patterns to exclude
    pub exclude: Vec<String>,
    /// Language-specific configurations
    pub languages: HashMap<String, LanguageConfig>,
}

/// Configuration for a specific language
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LanguageConfig {
    /// Whether this language is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Rule configurations
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,
}

fn default_true() -> bool {
    true
}

/// Configuration for a specific rule
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    /// Simple enable/disable
    Enabled(bool),
    /// Enable with "enable" keyword
    Enable(String),
    /// Detailed configuration
    Detailed(HashMap<String, serde_yaml_ng::Value>),
}

impl RuleConfig {
    /// Check if the rule is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            RuleConfig::Enabled(enabled) => *enabled,
            RuleConfig::Enable(s) => s == "enable",
            RuleConfig::Detailed(map) => {
                map.get("enable").and_then(|v| v.as_bool()).unwrap_or(true)
            }
        }
    }

    /// Get an option value
    pub fn get_option<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        match self {
            RuleConfig::Detailed(map) => map
                .get(key)
                .and_then(|v| serde_yaml_ng::from_value(v.clone()).ok()),
            _ => None,
        }
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        RuleConfig::Enabled(true)
    }
}

impl Config {
    /// Create a new config with the given root directory
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            ..Default::default()
        }
    }

    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = serde_yaml_ng::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Get the configuration for a specific language
    pub fn language_config(&self, lang: &str) -> Option<&LanguageConfig> {
        self.languages.get(lang)
    }

    /// Get the configuration for a specific rule in a language
    pub fn rule_config(&self, lang: &str, rule: &str) -> Option<&RuleConfig> {
        self.languages.get(lang).and_then(|lc| lc.rules.get(rule))
    }

    /// Check if a rule is enabled
    pub fn is_rule_enabled(&self, lang: &str, rule: &str) -> bool {
        self.rule_config(lang, rule)
            .map(|rc| rc.is_enabled())
            .unwrap_or(true) // Default to enabled
    }
}
