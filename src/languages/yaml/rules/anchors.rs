use std::collections::{HashMap, HashSet};

use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

pub struct Anchors {
    pub forbid_undeclared_aliases: bool,
    pub forbid_duplicated_anchors: bool,
    pub forbid_unused_anchors: bool,
}

impl Default for Anchors {
    fn default() -> Self {
        Self {
            forbid_undeclared_aliases: true,
            forbid_duplicated_anchors: true,
            forbid_unused_anchors: true,
        }
    }
}

impl Rule for Anchors {
    fn name(&self) -> &'static str {
        "anchors"
    }

    fn description(&self) -> &'static str {
        "Check anchors and aliases"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let forbid_undeclared_aliases = config
            .and_then(|c| c.get_option("forbid-undeclared-aliases"))
            .unwrap_or(self.forbid_undeclared_aliases);
        let forbid_duplicated_anchors = config
            .and_then(|c| c.get_option("forbid-duplicated-anchors"))
            .unwrap_or(self.forbid_duplicated_anchors);
        let forbid_unused_anchors = config
            .and_then(|c| c.get_option("forbid-unused-anchors"))
            .unwrap_or(self.forbid_unused_anchors);

        let mut diagnostics = Vec::new();
        let mut anchors: HashMap<String, Location> = HashMap::new();
        let mut aliases: Vec<(String, Location)> = Vec::new();

        for (idx, line) in ctx.lines.iter().enumerate() {
            let line_num = idx + 1;
            let mut in_quote = false;
            let mut quote_char = ' ';
            let mut chars = line.char_indices().peekable();

            while let Some((i, c)) = chars.next() {
                match c {
                    '"' | '\'' if !in_quote => {
                        in_quote = true;
                        quote_char = c;
                    }
                    c if c == quote_char && in_quote => {
                        in_quote = false;
                    }
                    '&' if !in_quote => {
                        // Found anchor
                        let start = i + 1;
                        let mut end = start;
                        while let Some(&(_, nc)) = chars.peek() {
                            if nc.is_alphanumeric() || nc == '_' || nc == '-' {
                                chars.next();
                                end += 1;
                            } else {
                                break;
                            }
                        }
                        if end > start {
                            let name = line[start..end].to_string();
                            let loc = Location::new(line_num, i + 1);
                            match anchors.entry(name) {
                                std::collections::hash_map::Entry::Occupied(entry) => {
                                    if forbid_duplicated_anchors {
                                        diagnostics.push(Diagnostic::error(
                                            self.name(),
                                            format!("found duplicated anchor \"{}\"", entry.key()),
                                            loc,
                                        ));
                                    }
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    entry.insert(loc);
                                }
                            }
                        }
                    }
                    '*' if !in_quote => {
                        // Found alias
                        let start = i + 1;
                        let mut end = start;
                        while let Some(&(_, nc)) = chars.peek() {
                            if nc.is_alphanumeric() || nc == '_' || nc == '-' {
                                chars.next();
                                end += 1;
                            } else {
                                break;
                            }
                        }
                        if end > start {
                            let name = line[start..end].to_string();
                            let loc = Location::new(line_num, i + 1);
                            aliases.push((name, loc));
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut used_anchors = HashSet::new();

        for (name, loc) in aliases {
            if !anchors.contains_key(&name) {
                if forbid_undeclared_aliases {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        format!("found undeclared alias \"{}\"", name),
                        loc,
                    ));
                }
            } else {
                used_anchors.insert(name);
            }
        }

        if forbid_unused_anchors {
            for (name, loc) in anchors {
                if !used_anchors.contains(&name) {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        format!("found unused anchor \"{}\"", name),
                        loc,
                    ));
                }
            }
        }

        diagnostics.sort_by_key(|d| (d.location.line, d.location.column));
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
    fn test_undeclared_alias() {
        let content = "alias: *undeclared\n";
        let ctx = RuleContext::new(content);
        let rule = Anchors::default();
        let diagnostics = rule.check(&ctx, None);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("undeclared"));
    }

    #[test]
    fn test_duplicated_anchor() {
        let content = "a: &anchor val1\nb: &anchor val2\nc: *anchor\n";
        let ctx = RuleContext::new(content);
        let rule = Anchors::default();
        let diagnostics = rule.check(&ctx, None);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("duplicated"));
    }

    #[test]
    fn test_unused_anchor() {
        let content = "a: &unused val\n";
        let ctx = RuleContext::new(content);
        let rule = Anchors::default();
        let diagnostics = rule.check(&ctx, None);

        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].message.contains("unused"));
    }

    #[test]
    fn test_valid_anchors() {
        let content = "a: &anchor val\nb: *anchor\n";
        let ctx = RuleContext::new(content);
        let rule = Anchors::default();
        let diagnostics = rule.check(&ctx, None);

        assert!(diagnostics.is_empty());
    }
}
