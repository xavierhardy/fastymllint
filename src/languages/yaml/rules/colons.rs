use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::rule::{Rule, RuleContext};

#[derive(Default)]
pub struct Colons {
    pub max_spaces_before: Option<usize>,
    pub max_spaces_after: Option<usize>,
}

impl Rule for Colons {
    fn name(&self) -> &'static str {
        "colons"
    }

    fn description(&self) -> &'static str {
        "Enforce consistant spacing around colons"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let content = &ctx.content;

        for (line_idx, line) in content.lines().enumerate() {
            let mut chars = line.chars().enumerate().peekable();

            while let Some((i, c)) = chars.next() {
                if c == ':' {
                    // Check spaces before
                    if let Some(max_before) = self.max_spaces_before {
                        let mut spaces_before = 0;
                        let mut j = i;
                        while j > 0 {
                            j -= 1;
                            if line.chars().nth(j) == Some(' ') {
                                spaces_before += 1;
                            } else {
                                break;
                            }
                        }

                        if spaces_before > max_before {
                            diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                message: "too many spaces before colon".to_string(),
                                location: Location {
                                    line: line_idx + 1,
                                    column: i + 1 - spaces_before,
                                },
                                end_location: Some(Location {
                                    line: line_idx + 1,
                                    column: i + 1,
                                }),
                                fix: None,
                                rule: self.name().to_string(),
                            });
                        }
                    }

                    // Check spaces after
                    if let Some(max_after) = self.max_spaces_after {
                        let mut spaces_after = 0;
                        // Peek forward
                        let mut iter = line[i + 1..].chars();
                        while let Some(next_char) = iter.next() {
                            if next_char == ' ' {
                                spaces_after += 1;
                            } else {
                                break;
                            }
                        }

                        if spaces_after > max_after {
                            diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                message: "too many spaces after colon".to_string(),
                                location: Location {
                                    line: line_idx + 1,
                                    column: i + 1 + 1,
                                },
                                end_location: Some(Location {
                                    line: line_idx + 1,
                                    column: i + 1 + 1 + spaces_after,
                                }),
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
    fn test_colons_before() {
        let rule = Colons {
            max_spaces_before: Some(0),
            ..Default::default()
        };
        let content = "key : value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too many spaces before colon");
    }

    #[test]
    fn test_colons_after() {
        let rule = Colons {
            max_spaces_after: Some(1),
            ..Default::default()
        };
        let content = "key:  value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too many spaces after colon");
    }

    #[test]
    fn test_colons_ok() {
        let rule = Colons {
            max_spaces_before: Some(0),
            max_spaces_after: Some(1),
            ..Default::default()
        };
        let content = "key: value";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 0);
    }
}
