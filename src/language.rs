//! Language trait for extensible language support

use std::path::Path;
use anyhow::Result;
use crate::{Config, Diagnostic};

/// Trait for language-specific linting, fixing, and formatting
pub trait Language: Send + Sync {
    /// Returns the name of this language (e.g., "yaml", "json")
    fn name(&self) -> &'static str;
    
    /// Returns file extensions this language handles (without leading dot)
    fn file_extensions(&self) -> &[&'static str];
    
    /// Detects if a file should be handled by this language
    fn detect(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            self.file_extensions().iter().any(|&e| e.eq_ignore_ascii_case(ext))
        } else {
            false
        }
    }
    
    /// Lint the content and return diagnostics
    fn lint(&self, content: &str, config: &Config) -> Vec<Diagnostic>;
    
    /// Apply fixes to the content and return the fixed version
    fn fix(&self, content: &str, config: &Config) -> Result<String>;
    
    /// Format the content according to the language's style rules
    fn format(&self, content: &str, config: &Config) -> Result<String>;
}

/// A boxed language handler
pub type BoxedLanguage = Box<dyn Language>;
