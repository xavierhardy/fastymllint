use crate::diagnostic::{Diagnostic, Fix, Location};
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

    fn check(
        &self,
        ctx: &RuleContext,
        _config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                // It's a full-line comment
                let indent = line.len() - trimmed.len();

                // Find next non-comment line to compare indentation
                let mut target_indent = None;

                // Look forward
                for next_line in ctx.lines.iter().skip(i + 1) {
                    let next_trimmed = next_line.trim_start();
                    if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                        target_indent = Some(next_line.len() - next_trimmed.len());
                        break;
                    }
                }

                // If no next line, look backward (e.g. comment at end of block)
                if target_indent.is_none() {
                    for prev_line in ctx.lines.iter().take(i).rev() {
                        let prev_trimmed = prev_line.trim_start();
                        if !prev_trimmed.is_empty() && !prev_trimmed.starts_with('#') {
                            target_indent = Some(prev_line.len() - prev_trimmed.len());
                            break;
                        }
                    }
                }

                if let Some(target) = target_indent
                    && indent != target
                {
                    let diag = Diagnostic::error(
                        self.name(),
                        "comment not indented like content",
                        Location::new(line_num, 1),
                    );
                    let new_line = format!("{}{}", " ".repeat(target), trimmed);
                    diagnostics.push(diag.with_fix(Fix::new(
                        "re-indent comment",
                        new_line,
                        Location::new(line_num, 1),
                        Location::new(line_num, line.len() + 1),
                    )));
                }
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        true
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
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_comments_indentation_invalid() {
        let rule = CommentsIndentation;
        let content = "key:\n  value\n# comment\n  other: value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "comment not indented like content");
    }
}
