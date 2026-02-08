//! YAML language support with yamllint-compatible configuration

pub mod config;
pub mod rules;

use anyhow::Result;
use std::path::Path;

use crate::rule::RuleContext;
use crate::{Config, Diagnostic, Language};
use rules::RuleSet;

/// YAML language handler
pub struct YamlLanguage {
    rules: RuleSet,
}

impl YamlLanguage {
    pub fn new() -> Self {
        Self {
            rules: RuleSet::default(),
        }
    }
}

impl Default for YamlLanguage {
    fn default() -> Self {
        Self::new()
    }
}

impl Language for YamlLanguage {
    fn name(&self) -> &'static str {
        "yaml"
    }

    fn file_extensions(&self) -> &[&'static str] {
        &["yaml", "yml"]
    }

    fn detect(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(ext.to_lowercase().as_str(), "yaml" | "yml")
        } else {
            // Also check for common YAML config files without extension
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(".yamllint"))
                .unwrap_or(false)
        }
    }

    fn lint(&self, content: &str, config: &Config) -> Vec<Diagnostic> {
        let ctx = RuleContext::new(content);
        let yaml_config = config.language_config("yaml");

        self.rules.check(&ctx, yaml_config)
    }

    fn fix(&self, content: &str, config: &Config) -> Result<String> {
        let mut result = content.to_string();
        let yaml_config = config.language_config("yaml");

        // Apply fixes in order of priority
        result = self.rules.fix(&result, yaml_config)?;

        Ok(result)
    }

    fn format(&self, content: &str, config: &Config) -> Result<String> {
        // Format is essentially a more aggressive fix
        self.fix(content, config)
    }
}
