use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

pub struct Commas {
    pub max_spaces_before: i64,
    pub min_spaces_after: i64,
    pub max_spaces_after: i64,
}

impl Default for Commas {
    fn default() -> Self {
        Self {
            max_spaces_before: 0,
            min_spaces_after: 1,
            max_spaces_after: 1,
        }
    }
}

impl Rule for Commas {
    fn name(&self) -> &'static str {
        "commas"
    }

    fn description(&self) -> &'static str {
        "Enforce consistent spacing around commas"
    }

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let max_before = config.and_then(|c| c.get_int("max-spaces-before")).unwrap_or(self.max_spaces_before);
        let min_after = config.and_then(|c| c.get_int("min-spaces-after")).unwrap_or(self.min_spaces_after);
        let max_after = config.and_then(|c| c.get_int("max-spaces-after")).unwrap_or(self.max_spaces_after);

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
                    ',' if !in_quote => {
                        // Spaces before
                        let mut spaces_before = 0;
                        let mut j = i;
                        while j > 0 && chars[j-1].1 == ' ' {
                            spaces_before += 1;
                            j -= 1;
                        }

                        if max_before >= 0 && (spaces_before as i64) > max_before {
                            let diag = Diagnostic::error(
                                self.name(),
                                "too many spaces before comma",
                                Location::new(line_num, chars[j].0 + 1),
                            );
                            diagnostics.push(diag.with_fix(Fix::delete(
                                "remove extra spaces before comma",
                                Location::new(line_num, chars[j].0 + 1),
                                Location::new(line_num, pos + 1),
                            )));
                        }

                        // Spaces after
                        let mut spaces_after = 0;
                        let mut k = i + 1;
                        while k < chars.len() && chars[k].1 == ' ' {
                            spaces_after += 1;
                            k += 1;
                        }
                        
                        let next_char = if k < chars.len() { Some(chars[k].1) } else { None };

                        if min_after >= 0 && (spaces_after as i64) < min_after && next_char.is_some() && next_char != Some('\n') {
                             let diag = Diagnostic::error(
                                self.name(),
                                "too few spaces after comma",
                                Location::new(line_num, pos + 1),
                            );
                            diagnostics.push(diag.with_fix(Fix::insert(
                                "add spaces after comma",
                                " ".repeat((min_after - (spaces_after as i64)) as usize),
                                Location::new(line_num, pos + 2),
                            )));
                        } else if max_after >= 0 && (spaces_after as i64) > max_after {
                            let diag = Diagnostic::error(
                                self.name(),
                                "too many spaces after comma",
                                Location::new(line_num, pos + 2),
                            );
                            diagnostics.push(diag.with_fix(Fix::delete(
                                "remove extra spaces after comma",
                                Location::new(line_num, pos + 2),
                                Location::new(line_num, pos + 2 + (spaces_after - max_after as usize)),
                            )));
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
    fn test_commas_before() {
        let rule = Commas {
            max_spaces_before: 0,
            ..Default::default()
        };
        let content = "[item1 , item2]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too many spaces before comma");
    }

    #[test]
    fn test_commas_min_after() {
        let rule = Commas {
            min_spaces_after: 1,
            ..Default::default()
        };
        let content = "[item1,item2]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too few spaces after comma");
    }

    #[test]
    fn test_commas_max_after() {
        let rule = Commas {
            max_spaces_after: 1,
            ..Default::default()
        };
        let content = "[item1,  item2]";
        let ctx = RuleContext::new(content);
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "too many spaces after comma");
    }
}
