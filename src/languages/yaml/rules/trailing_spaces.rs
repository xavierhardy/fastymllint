//! Trailing spaces rule

use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that checks for trailing whitespace on lines
pub struct TrailingSpaces;

impl Rule for TrailingSpaces {
    fn name(&self) -> &'static str {
        "trailing-spaces"
    }

    fn description(&self) -> &'static str {
        "Forbid trailing spaces at the end of lines"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        _config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        ctx.lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let trimmed = line.trim_end();
                if trimmed.len() < line.len() {
                    let line_num = idx + 1;
                    let col = trimmed.len() + 1;
                    Some(
                        Diagnostic::warning(
                            self.name(),
                            "trailing spaces",
                            Location::new(line_num, col),
                        )
                        .with_fix(Fix::delete(
                            "Remove trailing spaces",
                            Location::new(line_num, col),
                            Location::new(line_num, line.len() + 1),
                        )),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_fixable(&self) -> bool {
        true
    }
}

/// Fix trailing spaces in content
pub fn fix_trailing_spaces(content: &str) -> String {
    content
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        + if content.ends_with('\n') { "\n" } else { "" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trailing_spaces_detection() {
        let content = "hello   \nworld\nfoo  ";
        let ctx = RuleContext::new(content);
        let diagnostics = TrailingSpaces.check(&ctx, None);

        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].location.line, 1);
        assert_eq!(diagnostics[1].location.line, 3);
    }

    #[test]
    fn test_fix_trailing_spaces() {
        let content = "hello   \nworld\nfoo  \n";
        let fixed = fix_trailing_spaces(content);
        assert_eq!(fixed, "hello\nworld\nfoo\n");
    }
}
