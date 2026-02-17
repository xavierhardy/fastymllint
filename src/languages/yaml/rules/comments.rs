use crate::diagnostic::{Diagnostic, Fix, Location};
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

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let require_starting_space = config
            .and_then(|c| c.get_option("require-starting-space"))
            .unwrap_or(self.require_starting_space);
        let ignore_shebangs = config
            .and_then(|c| c.get_option("ignore-shebangs"))
            .unwrap_or(self.ignore_shebangs);
        let min_spaces_from_content = config
            .and_then(|c| c.get_option("min-spaces-from-content"))
            .unwrap_or(self.min_spaces_from_content);

        let mut diagnostics = Vec::new();

        for (idx, line) in ctx.lines.iter().enumerate() {
            let line_num = idx + 1;
            if let Some(comment_start) = line.find('#') {
                // Ignore shebangs if configured
                if ignore_shebangs && line_num == 1 && line.starts_with("#!") {
                    continue;
                }

                // Check for space after comment marker
                if require_starting_space
                    && let Some(next_char) = line[comment_start + 1..].chars().next()
                    && next_char != ' '
                    && !next_char.is_whitespace()
                {
                    let diag = Diagnostic::error(
                        self.name(),
                        "no space after comment's #",
                        Location::new(line_num, comment_start + 1),
                    );
                    diagnostics.push(diag.with_fix(Fix::insert(
                        "add space after #",
                        " ",
                        Location::new(line_num, comment_start + 2),
                    )));
                }

                // Check for spaces before inline comments
                if comment_start > 0 {
                    let before_comment = &line[..comment_start];
                    if !before_comment.trim().is_empty() {
                        // It's an inline comment
                        let mut spaces = 0;
                        for c in before_comment.chars().rev() {
                            if c == ' ' {
                                spaces += 1;
                            } else {
                                break;
                            }
                        }

                        if spaces < min_spaces_from_content {
                            let diag = Diagnostic::error(
                                self.name(),
                                "too few spaces before comment",
                                Location::new(line_num, comment_start + 1 - spaces),
                            );
                            diagnostics.push(diag.with_fix(Fix::insert(
                                "add spaces before comment",
                                " ".repeat(min_spaces_from_content - spaces),
                                Location::new(line_num, comment_start + 1),
                            )));
                        }
                    }
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
    fn test_comments_starting_space() {
        let rule = Comments {
            require_starting_space: true,
            ..Default::default()
        };
        let content = "#comment";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "no space after comment's #");
    }

    #[test]
    fn test_comments_min_spaces() {
        let rule = Comments {
            min_spaces_from_content: 2,
            require_starting_space: false,
            ignore_shebangs: true,
        };
        let content = "key: value #comment";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too few spaces before comment");
    }

    #[test]
    fn test_comments_shebang() {
        let rule = Comments {
            ..Default::default()
        };
        let content = "#!/bin/bash";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 0);
    }
}
