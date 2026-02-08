//! Line length rule

use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that checks for lines exceeding maximum length
pub struct LineLength {
    max: usize,
    allow_non_breakable_words: bool,
    allow_non_breakable_inline_mappings: bool,
}

impl LineLength {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        }
    }
    
    pub fn with_options(
        max: usize,
        allow_non_breakable_words: bool,
        allow_non_breakable_inline_mappings: bool,
    ) -> Self {
        Self {
            max,
            allow_non_breakable_words,
            allow_non_breakable_inline_mappings,
        }
    }
    
    fn is_non_breakable_word(&self, line: &str) -> bool {
        // Check if the line contains a single long word (like a URL)
        let trimmed = line.trim();
        
        // If the line has no spaces after trimming leading content, it's non-breakable
        if let Some(pos) = trimmed.rfind(|c: char| c.is_whitespace() || c == ':') {
            let after_space = &trimmed[pos + 1..];
            // The part after the last space is longer than max and has no spaces
            after_space.len() > self.max && !after_space.contains(' ')
        } else {
            // Entire line is one word
            trimmed.len() > self.max && !trimmed.contains(' ')
        }
    }
    
    fn is_inline_mapping(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.contains(": ") && !trimmed.starts_with('-') && !trimmed.starts_with('#')
    }
}

impl Default for LineLength {
    fn default() -> Self {
        Self::new(80)
    }
}

impl Rule for LineLength {
    fn name(&self) -> &'static str {
        "line-length"
    }
    
    fn description(&self) -> &'static str {
        "Limit line length to a maximum number of characters"
    }
    
    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        ctx.lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let len = line.len();
                if len > self.max {
                    // Check exceptions
                    if self.allow_non_breakable_words && self.is_non_breakable_word(line) {
                        return None;
                    }
                    if self.allow_non_breakable_inline_mappings && self.is_inline_mapping(line) {
                        return None;
                    }
                    
                    let line_num = idx + 1;
                    Some(Diagnostic::warning(
                        self.name(),
                        format!("line too long ({} > {})", len, self.max),
                        Location::new(line_num, self.max + 1),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }
    
    fn is_fixable(&self) -> bool {
        false // Line length is not auto-fixable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_line_length_detection() {
        let content = "short\nthis is a very long line that exceeds the maximum length limit we have set\nok";
        let ctx = RuleContext::new(content);
        let rule = LineLength::new(40);
        let diagnostics = rule.check(&ctx);
        
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].location.line, 2);
    }
    
    #[test]
    fn test_allow_non_breakable_words() {
        let content = "url: https://very-long-url.example.com/with/a/very/long/path/that/exceeds/limit";
        let ctx = RuleContext::new(content);
        let rule = LineLength::with_options(40, true, false);
        let diagnostics = rule.check(&ctx);
        
        assert!(diagnostics.is_empty());
    }
}
