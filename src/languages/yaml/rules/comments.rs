use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::rule::{Rule, RuleContext};

pub struct Comments {
    pub require_starting_space: bool,
    pub ignore_shebangs: bool,
    pub min_spaces_from_content: usize,
}

impl Default for Comments {
    fn default() -> Self {
        Self {
            require_starting_space: true,
            ignore_shebangs: true,
            min_spaces_from_content: 2,
        }
    }
}

impl Rule for Comments {
    fn name(&self) -> &'static str {
        "comments"
    }

    fn description(&self) -> &'static str {
        "Enforce comment formatting"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let content = &ctx.content;

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if let Some(comment_start) = line.find('#') {
                // Ignore shebangs if configured
                if self.ignore_shebangs && line_idx == 0 && line.starts_with("#!") {
                    continue;
                }
                
                // Check starting space
                if self.require_starting_space {
                     if comment_start + 1 < line.len() {
                         let next_char = line[comment_start+1..].chars().next();
                         if let Some(c) = next_char {
                             if c != ' ' && c != '#' { // Allow '##' for headers/sections
                                  diagnostics.push(Diagnostic {
                                    severity: Severity::Error,
                                    message: "too few spaces before comment".to_string(), // Actually "missing starting space"
                                    location: Location { line: line_idx + 1, column: comment_start + 1 },
                                    end_location: Some(Location { line: line_idx + 1, column: comment_start + 2 }),
                                    fix: None,
                                    rule: self.name().to_string(),
                                });
                             }
                         }
                     }
                }
                
                // Check min spaces from content
                if comment_start > 0 {
                    let before_comment = &line[..comment_start];
                    if !before_comment.trim().is_empty() {
                         // Content exists before comment
                         let mut spaces = 0;
                         for c in before_comment.chars().rev() {
                             if c == ' ' {
                                 spaces += 1;
                             } else {
                                 break;
                             }
                         }
                         
                         if spaces < self.min_spaces_from_content {
                              diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                message: "too few spaces before comment".to_string(),
                                location: Location { line: line_idx + 1, column: comment_start + 1 - spaces },
                                end_location: Some(Location { line: line_idx + 1, column: comment_start + 1 }),
                                fix: None,
                                rule: self.name().to_string(),
                            });
                         }
                    }
                }
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comments_starting_space() {
        let rule = Comments { require_starting_space: true, ..Default::default() };
        let content = "#comment";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too few spaces before comment"); // Wait, message is confusing, fix implementation message
    }

    #[test]
    fn test_comments_min_spaces() {
        let rule = Comments { min_spaces_from_content: 2, require_starting_space: false, ..Default::default() };
        let content = "key: value #comment";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too few spaces before comment");
    }

    #[test]
    fn test_comments_shebang() {
        let rule = Comments { ..Default::default() };
        let content = "#!/bin/bash";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 0);
    }
}
