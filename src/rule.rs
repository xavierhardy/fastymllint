//! Rule trait for extensible lint rules

use crate::Diagnostic;

/// Context provided to rules for checking content
#[derive(Debug)]
pub struct RuleContext<'a> {
    /// The file content as a string
    pub content: &'a str,
    /// Lines of the content (for line-by-line checking)
    pub lines: Vec<&'a str>,
    /// The file path (if available)
    pub file_path: Option<&'a str>,
}

impl<'a> RuleContext<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            lines: content.lines().collect(),
            file_path: None,
        }
    }

    pub fn with_path(mut self, path: &'a str) -> Self {
        self.file_path = Some(path);
        self
    }

    /// Get a specific line (1-indexed)
    pub fn line(&self, n: usize) -> Option<&'a str> {
        self.lines.get(n.saturating_sub(1)).copied()
    }

    /// Get the total number of lines
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }
}

/// Trait for individual lint rules
pub trait Rule: Send + Sync {
    /// Returns the unique name/identifier of this rule
    fn name(&self) -> &'static str;

    /// Returns a brief description of what this rule checks
    fn description(&self) -> &'static str;

    /// Check the content and return any diagnostics
    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic>;

    /// Whether this rule can automatically fix issues
    fn is_fixable(&self) -> bool {
        false
    }
}

/// A boxed rule
pub type BoxedRule = Box<dyn Rule>;
