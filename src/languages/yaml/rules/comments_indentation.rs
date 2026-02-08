use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::rule::{Rule, RuleContext};

#[derive(Default)]
pub struct CommentsIndentation;

impl Rule for CommentsIndentation {
    fn name(&self) -> &'static str {
        "comments-indentation"
    }

    fn description(&self) -> &'static str {
        "Enforce comment indentation consistency"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let content = &ctx.content;
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                // It's a full-line comment
                let indent = line.len() - trimmed.len();

                // Find next non-comment line to compare indentation
                let mut target_indent = None;

                // Look forward
                for next_line in lines.iter().skip(i + 1) {
                    let next_trimmed = next_line.trim_start();
                    if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                        target_indent = Some(next_line.len() - next_trimmed.len());
                        break;
                    }
                }

                // If no next line, look backward (e.g. comment at end of block)
                if target_indent.is_none() {
                    for prev_line in lines.iter().take(i).rev() {
                        let prev_trimmed = prev_line.trim_start();
                        if !prev_trimmed.is_empty() && !prev_trimmed.starts_with('#') {
                            target_indent = Some(prev_line.len() - prev_trimmed.len());
                            break;
                        }
                    }
                }

                if let Some(target) = target_indent {
                    if indent != target {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            message: "comment not indented like content".to_string(),
                            location: Location {
                                line: i + 1,
                                column: 1,
                            },
                            end_location: Some(Location {
                                line: i + 1,
                                column: indent + 1,
                            }),
                            fix: None,
                            rule: self.name().to_string(),
                        });
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
    fn test_comments_indentation_valid() {
        let rule = CommentsIndentation;
        let content = "key:\n  value\n  # comment\n  other: value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_comments_indentation_invalid() {
        let rule = CommentsIndentation;
        let content = "key:\n  value\n# comment\n  other: value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "comment not indented like content");
    }
}
