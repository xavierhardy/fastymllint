use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

#[derive(Default)]
pub struct Brackets {
    pub min_spaces_inside: usize,
    pub max_spaces_inside: usize,
    pub min_spaces_inside_empty: Option<usize>,
    pub max_spaces_inside_empty: Option<usize>,
    pub forbid: bool,
    pub forbid_non_empty: bool,
}

impl Rule for Brackets {
    fn name(&self) -> &'static str {
        "brackets"
    }

    fn description(&self) -> &'static str {
        "Enforce consistent spacing inside brackets"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let min_spaces_inside = config
            .and_then(|c| c.get_option("min-spaces-inside"))
            .unwrap_or(self.min_spaces_inside);
        let max_spaces_inside = config
            .and_then(|c| c.get_option("max-spaces-inside"))
            .unwrap_or(self.max_spaces_inside);
        let min_spaces_inside_empty = config
            .and_then(|c| c.get_option("min-spaces-inside-empty"))
            .or(self.min_spaces_inside_empty);
        let max_spaces_inside_empty = config
            .and_then(|c| c.get_option("max-spaces-inside-empty"))
            .or(self.max_spaces_inside_empty);
        let forbid = config
            .and_then(|c| c.get_option("forbid"))
            .unwrap_or(self.forbid);
        let forbid_non_empty = config
            .and_then(|c| c.get_option("forbid-non-empty"))
            .unwrap_or(self.forbid_non_empty);

        let mut diagnostics = Vec::new();

        for (idx, line) in ctx.lines.iter().enumerate() {
            let line_num = idx + 1;
            let mut in_quote = false;
            let mut quote_char = ' ';
            let chars: Vec<(usize, char)> = line.char_indices().collect();

            let mut i = 0;
            while i < chars.len() {
                let (pos, c) = chars[i];
                match c {
                    '"' | '\'' if !in_quote => {
                        in_quote = true;
                        quote_char = c;
                    }
                    c if c == quote_char && in_quote => {
                        in_quote = false;
                    }
                    '[' if !in_quote => {
                        if forbid {
                            diagnostics.push(Diagnostic::error(
                                self.name(),
                                "flow sequences are forbidden",
                                Location::new(line_num, pos + 1),
                            ));
                        } else {
                            // Check for empty brackets
                            let mut j = i + 1;
                            let mut spaces_after = 0;
                            while j < chars.len() && chars[j].1 == ' ' {
                                spaces_after += 1;
                                j += 1;
                            }

                            if j < chars.len() && chars[j].1 == ']' {
                                // Empty brackets
                                if forbid_non_empty {
                                    // OK, only non-empty forbidden
                                } else {
                                    let min_empty =
                                        min_spaces_inside_empty.unwrap_or(min_spaces_inside);
                                    let max_empty =
                                        max_spaces_inside_empty.unwrap_or(max_spaces_inside);

                                    if spaces_after < min_empty {
                                        let diag = Diagnostic::error(
                                            self.name(),
                                            "too few spaces inside empty brackets",
                                            Location::new(line_num, pos + 1),
                                        );
                                        diagnostics.push(diag.with_fix(Fix::insert(
                                            "add spaces inside empty brackets",
                                            " ".repeat(min_empty - spaces_after),
                                            Location::new(line_num, pos + 2),
                                        )));
                                    } else if spaces_after > max_empty {
                                        let diag = Diagnostic::error(
                                            self.name(),
                                            "too many spaces inside empty brackets",
                                            Location::new(line_num, pos + 2),
                                        );
                                        diagnostics.push(diag.with_fix(Fix::delete(
                                            "remove extra spaces inside empty brackets",
                                            Location::new(line_num, pos + 2),
                                            Location::new(
                                                line_num,
                                                pos + 2 + (spaces_after - max_empty),
                                            ),
                                        )));
                                    }
                                }
                                i = j; // Skip to ]
                            } else {
                                // Non-empty brackets
                                if forbid_non_empty {
                                    diagnostics.push(Diagnostic::error(
                                        self.name(),
                                        "non-empty flow sequences are forbidden",
                                        Location::new(line_num, pos + 1),
                                    ));
                                }

                                if spaces_after < min_spaces_inside {
                                    let diag = Diagnostic::error(
                                        self.name(),
                                        "too few spaces inside brackets",
                                        Location::new(line_num, pos + 1),
                                    );
                                    diagnostics.push(diag.with_fix(Fix::insert(
                                        "add spaces inside brackets",
                                        " ".repeat(min_spaces_inside - spaces_after),
                                        Location::new(line_num, pos + 2),
                                    )));
                                } else if spaces_after > max_spaces_inside {
                                    let diag = Diagnostic::error(
                                        self.name(),
                                        "too many spaces inside brackets",
                                        Location::new(line_num, pos + 2),
                                    );
                                    diagnostics.push(diag.with_fix(Fix::delete(
                                        "remove extra spaces inside brackets",
                                        Location::new(line_num, pos + 2),
                                        Location::new(
                                            line_num,
                                            pos + 2 + (spaces_after - max_spaces_inside),
                                        ),
                                    )));
                                }
                            }
                        }
                    }
                    ']' if !in_quote => {
                        let mut j = pos;
                        let mut spaces_before = 0;
                        while j > 0 && line.as_bytes()[j - 1] == b' ' {
                            spaces_before += 1;
                            j -= 1;
                        }

                        let mut is_empty = false;
                        if j > 0 && line.as_bytes()[j - 1] == b'[' {
                            is_empty = true;
                        }

                        if !is_empty && !forbid && !forbid_non_empty {
                            if spaces_before < min_spaces_inside {
                                let diag = Diagnostic::error(
                                    self.name(),
                                    "too few spaces inside brackets",
                                    Location::new(line_num, pos + 1),
                                );
                                diagnostics.push(diag.with_fix(Fix::insert(
                                    "add spaces inside brackets",
                                    " ".repeat(min_spaces_inside - spaces_before),
                                    Location::new(line_num, pos + 1),
                                )));
                            } else if spaces_before > max_spaces_inside {
                                let diag = Diagnostic::error(
                                    self.name(),
                                    "too many spaces inside brackets",
                                    Location::new(
                                        line_num,
                                        pos + 1 - (spaces_before - max_spaces_inside),
                                    ),
                                );
                                diagnostics.push(diag.with_fix(Fix::delete(
                                    "remove extra spaces inside brackets",
                                    Location::new(
                                        line_num,
                                        pos + 1 - (spaces_before - max_spaces_inside),
                                    ),
                                    Location::new(line_num, pos + 1),
                                )));
                            }
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
        }

        diagnostics.sort_by_key(|d| (d.location.line, d.location.column));
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
    fn test_brackets_forbid() {
        let rule = Brackets {
            forbid: true,
            ..Default::default()
        };
        let content = "[item1, item2]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "flow sequences are forbidden");
    }

    #[test]
    fn test_brackets_min_spaces() {
        let rule = Brackets {
            min_spaces_inside: 1,
            ..Default::default()
        };
        let content = "[item1, item2]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 2);
        assert_eq!(diagnostics[0].message, "too few spaces inside brackets");
    }
}
