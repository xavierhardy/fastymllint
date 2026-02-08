use crate::diagnostic::{Diagnostic, Location, Severity};
use crate::rule::{Rule, RuleContext};

pub struct Brackets {
    min_spaces_inside: usize,
    max_spaces_inside: usize,
    min_spaces_inside_empty: Option<usize>,
    max_spaces_inside_empty: Option<usize>,
    forbid: bool,
    forbid_non_empty: bool,
}

impl Default for Brackets {
    fn default() -> Self {
        Self {
            min_spaces_inside: 0,
            max_spaces_inside: 0,
            min_spaces_inside_empty: None,
            max_spaces_inside_empty: None,
            forbid: false,
            forbid_non_empty: false,
        }
    }
}

impl Rule for Brackets {
    fn name(&self) -> &'static str {
        "brackets"
    }

    fn description(&self) -> &'static str {
        "Enforce consistant spacing inside brackets"
    }

    fn check(&self, ctx: &RuleContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let content = &ctx.content;

        for (line_idx, line) in content.lines().enumerate() {
            let mut i = 0;
            while i < line.len() {
                 if let Some(c) = line[i..].chars().next() {
                    if c == '[' {
                        if self.forbid {
                             diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                message: "flow sequences are forbidden".to_string(),
                                location: Location {
                                    line: line_idx + 1,
                                    column: i + 1,
                                },
                                end_location: Some(Location {
                                     line: line_idx + 1,
                                     column: i + 1 + c.len_utf8(),
                                }),
                                fix: None,
                                rule: self.name().to_string(),
                            });
                        }
                        
                        if !self.forbid {
                             let rest_of_line = &line[i+c.len_utf8()..];
                             if let Some(next_char) = rest_of_line.chars().next() {
                                 if next_char != ' ' && self.min_spaces_inside > 0 {
                                     if next_char == ']' {
                                         if let Some(min_empty) = self.min_spaces_inside_empty {
                                             if min_empty > 0 {
                                                  diagnostics.push(Diagnostic {
                                                    severity: Severity::Error,
                                                    message: "too few spaces inside empty brackets".to_string(),
                                                    location: Location { line: line_idx + 1, column: i + 1 },
                                                    end_location: Some(Location { line: line_idx + 1, column: i + 1 + c.len_utf8() }),
                                                    fix: None,
                                                    rule: self.name().to_string(),
                                                });
                                             }
                                         }
                                     } else {
                                          diagnostics.push(Diagnostic {
                                            severity: Severity::Error,
                                            message: "too few spaces inside brackets".to_string(),
                                            location: Location { line: line_idx + 1, column: i + 1 },
                                            end_location: Some(Location { line: line_idx + 1, column: i + 1 + c.len_utf8() }),
                                            fix: None,
                                            rule: self.name().to_string(),
                                        });
                                     }
                                 }
                             }
                        }
                    }
                    i += c.len_utf8();
                 } else {
                     break;
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
    fn test_brackets_forbid() {
        let rule = Brackets { forbid: true, ..Default::default() };
        let content = "[item]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "flow sequences are forbidden");
    }

    #[test]
    fn test_brackets_min_spaces() {
        let rule = Brackets { min_spaces_inside: 1, ..Default::default() };
        let content = "[item]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too few spaces inside brackets");
    }
}
