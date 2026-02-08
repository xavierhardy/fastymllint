//! Language registry and handlers

pub mod yaml;

use std::path::Path;
use crate::language::BoxedLanguage;

/// Registry of all supported languages
pub struct LanguageRegistry {
    languages: Vec<BoxedLanguage>,
}

impl LanguageRegistry {
    /// Create a new registry with all built-in languages
    pub fn new() -> Self {
        Self {
            languages: vec![
                Box::new(yaml::YamlLanguage::new()),
            ],
        }
    }
    
    /// Detect which language handles a file
    pub fn detect(&self, path: &Path) -> Option<&dyn crate::Language> {
        self.languages
            .iter()
            .find(|lang| lang.detect(path))
            .map(|lang| lang.as_ref())
    }
    
    /// Get all registered languages
    pub fn languages(&self) -> &[BoxedLanguage] {
        &self.languages
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}
