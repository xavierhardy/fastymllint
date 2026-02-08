//! Indentation rule

use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that checks for consistent indentation
pub struct Indentation {
    /// Number of spaces per indentation level (or 0 for "consistent")
    spaces: usize,
    /// Whether block sequences should be indented
    indent_sequences: IndentSequences,
    /// Whether to check multi-line strings
    check_multi_line_strings: bool,
}

/// How to handle sequence indentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentSequences {
    /// Sequences must be indented
    True,
    /// Sequences must not be indented
    False,
    /// Either is acceptable
    Whatever,
    /// Must be consistent within the file
    Consistent,
}

impl Indentation {
    pub fn new(spaces: usize) -> Self {
        Self {
            spaces,
            indent_sequences: IndentSequences::True,
            check_multi_line_strings: false,
        }
    }
    
    pub fn with_options(
        spaces: usize,
        indent_sequences: IndentSequences,
        check_multi_line_strings: bool,
    ) -> Self {
        Self {
            spaces,
            indent_sequences,
            check_multi_line_strings,
        }
    }
    
    fn get_indentation(line: &str) -> usize {
        line.len() - line.trim_start().len()
    }
    
    fn is_comment_or_empty(line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.is_empty() || trimmed.starts_with('#')
    }
}

impl Default for Indentation {
    fn default() -> Self {
        Self {
            spaces: 2, // Will be treated as "consistent" check
            indent_sequences: IndentSequences::True,
            check_multi_line_strings: false,
        }
    }
}

impl Rule for Indentation {
    fn name(&self) -> &'static str {
        "indentation"
    }
    
    fn description(&self) -> &'static str {
        "Check for consistent indentation"
    }
    
    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut detected_indent: Option<usize> = None;
        let mut prev_indent = 0;
        
        for (idx, line) in ctx.lines.iter().enumerate() {
            if Self::is_comment_or_empty(line) {
                continue;
            }
            
            let current_indent = Self::get_indentation(line);
            let line_num = idx + 1;
            
            // Check if indentation increased
            if current_indent > prev_indent {
                let indent_diff = current_indent - prev_indent;
                
                // Detect or validate indentation size
                if let Some(expected) = detected_indent {
                    if indent_diff != expected && indent_diff != 0 {
                        diagnostics.push(Diagnostic::warning(
                            self.name(),
                            format!(
                                "wrong indentation: expected {} but found {}",
                                expected, indent_diff
                            ),
                            Location::new(line_num, 1),
                        ));
                    }
                } else if self.spaces == 0 {
                    // "Consistent" mode: detect first indentation
                    detected_indent = Some(indent_diff);
                } else if indent_diff != self.spaces {
                    diagnostics.push(Diagnostic::warning(
                        self.name(),
                        format!(
                            "wrong indentation: expected {} but found {}",
                            self.spaces, indent_diff
                        ),
                        Location::new(line_num, 1),
                    ));
                }
            }
            
            // Check for odd indentation (not a multiple of expected)
            let expected_spaces = detected_indent.unwrap_or(self.spaces);
            if expected_spaces > 0 && current_indent % expected_spaces != 0 {
                diagnostics.push(Diagnostic::warning(
                    self.name(),
                    format!(
                        "indentation is not a multiple of {}",
                        expected_spaces
                    ),
                    Location::new(line_num, 1),
                ));
            }
            
            prev_indent = current_indent;
        }
        
        diagnostics
    }
    
    fn is_fixable(&self) -> bool {
        false // Indentation fixing is complex and risky
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consistent_indentation() {
        let content = "root:\n  child1:\n    grandchild: value\n  child2: value\n";
        let ctx = RuleContext::new(content);
        let rule = Indentation::new(2);
        let diagnostics = rule.check(&ctx);
        
        assert!(diagnostics.is_empty());
    }
    
    #[test]
    fn test_inconsistent_indentation() {
        let content = "root:\n  child1:\n   bad_indent: value\n";
        let ctx = RuleContext::new(content);
        let rule = Indentation::new(2);
        let diagnostics = rule.check(&ctx);
        
        assert!(!diagnostics.is_empty());
    }
}
