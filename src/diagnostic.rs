//! Diagnostic types for reporting linting issues

use std::fmt;

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Informational hint
    Hint,
    /// Warning - code works but could be improved
    Warning,
    /// Error - code violates a rule
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Hint => write!(f, "hint"),
            Severity::Warning => write!(f, "warning"),
            Severity::Error => write!(f, "error"),
        }
    }
}

/// Location in a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A suggested fix for a diagnostic
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// Description of what the fix does
    pub description: String,
    /// The replacement text
    pub replacement: String,
    /// Start location of the text to replace
    pub start: Location,
    /// End location of the text to replace
    pub end: Location,
}

impl Fix {
    pub fn new(
        description: impl Into<String>,
        replacement: impl Into<String>,
        start: Location,
        end: Location,
    ) -> Self {
        Self {
            description: description.into(),
            replacement: replacement.into(),
            start,
            end,
        }
    }

    /// Create a fix that inserts text at a location
    pub fn insert(
        description: impl Into<String>,
        text: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            description: description.into(),
            replacement: text.into(),
            start: location,
            end: location,
        }
    }

    /// Create a fix that deletes text between two locations
    pub fn delete(description: impl Into<String>, start: Location, end: Location) -> Self {
        Self {
            description: description.into(),
            replacement: String::new(),
            start,
            end,
        }
    }
}

/// A diagnostic message from a linter rule
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The rule that generated this diagnostic
    pub rule: String,
    /// Severity of the issue
    pub severity: Severity,
    /// Human-readable message
    pub message: String,
    /// Location in the source file
    pub location: Location,
    /// Optional end location for multi-character/line issues
    pub end_location: Option<Location>,
    /// Optional fix suggestion
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn new(
        rule: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self {
            rule: rule.into(),
            severity,
            message: message.into(),
            location,
            end_location: None,
            fix: None,
        }
    }

    pub fn error(rule: impl Into<String>, message: impl Into<String>, location: Location) -> Self {
        Self::new(rule, Severity::Error, message, location)
    }

    pub fn warning(
        rule: impl Into<String>,
        message: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::new(rule, Severity::Warning, message, location)
    }

    pub fn with_end(mut self, end: Location) -> Self {
        self.end_location = Some(end);
        self
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: [{}] {}", self.location, self.rule, self.message)
    }
}
