//! New line at end of file rule

use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that requires files to end with a newline
pub struct NewLineAtEndOfFile;

impl Rule for NewLineAtEndOfFile {
    fn name(&self) -> &'static str {
        "new-line-at-end-of-file"
    }
    
    fn description(&self) -> &'static str {
        "Require a new line at the end of the file"
    }
    
    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        if ctx.content.is_empty() {
            return vec![];
        }
        
        if !ctx.content.ends_with('\n') {
            let line_count = ctx.line_count();
            let last_line_len = ctx.lines.last().map(|l| l.len()).unwrap_or(0);
            
            vec![
                Diagnostic::warning(
                    self.name(),
                    "no new line character at the end of file",
                    Location::new(line_count, last_line_len + 1),
                )
                .with_fix(Fix::insert(
                    "Add newline at end of file",
                    "\n",
                    Location::new(line_count, last_line_len + 1),
                )),
            ]
        } else {
            vec![]
        }
    }
    
    fn is_fixable(&self) -> bool {
        true
    }
}

/// Fix missing newline at end of file
pub fn fix_new_line_at_end(content: &str) -> String {
    if content.is_empty() || content.ends_with('\n') {
        content.to_string()
    } else {
        format!("{}\n", content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_missing_newline() {
        let content = "hello: world";
        let ctx = RuleContext::new(content);
        let diagnostics = NewLineAtEndOfFile.check(&ctx);
        
        assert_eq!(diagnostics.len(), 1);
    }
    
    #[test]
    fn test_has_newline() {
        let content = "hello: world\n";
        let ctx = RuleContext::new(content);
        let diagnostics = NewLineAtEndOfFile.check(&ctx);
        
        assert!(diagnostics.is_empty());
    }
    
    #[test]
    fn test_fix_newline() {
        assert_eq!(fix_new_line_at_end("foo"), "foo\n");
        assert_eq!(fix_new_line_at_end("foo\n"), "foo\n");
    }
}
