use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct LineLength {
    pub max: usize,
    pub allow_non_breakable_words: bool,
    pub allow_non_breakable_inline_mappings: bool,
}

impl Default for LineLength {
    fn default() -> Self {
        Self {
            max: 80,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        }
    }
}

impl Rule for LineLength {
    fn name(&self) -> &'static str {
        "line-length"
    }

    fn description(&self) -> &'static str {
        "Limit line length to a maximum number of characters"
    }

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let max = config.and_then(|c| c.get_option("max")).unwrap_or(self.max);
        let allow_non_breakable_words = config.and_then(|c| c.get_option("allow-non-breakable-words")).unwrap_or(self.allow_non_breakable_words);
        let allow_non_breakable_inline_mappings = config.and_then(|c| c.get_option("allow-non-breakable-inline-mappings")).unwrap_or(self.allow_non_breakable_inline_mappings);

        ctx.lines
            .iter()
            .enumerate()
            .filter_map(|(idx, line)| {
                let len = line.len();
                if len > max {
                    if allow_non_breakable_words && is_non_breakable_word(line, max) {
                        return None;
                    }
                    if allow_non_breakable_inline_mappings && is_inline_mapping(line) {
                        return None;
                    }

                    let line_num = idx + 1;
                    Some(Diagnostic::warning(
                        self.name(),
                        format!("line too long ({} > {})", len, max),
                        crate::diagnostic::Location::new(line_num, max + 1),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}

fn is_non_breakable_word(line: &str, max: usize) -> bool {
    let trimmed = line.trim();
    if let Some(pos) = trimmed.rfind(|c: char| c.is_whitespace() || c == ':') {
        let after_space = &trimmed[pos + 1..];
        after_space.len() > max && !after_space.contains(' ')
    } else {
        trimmed.len() > max && !trimmed.contains(' ')
    }
}

fn is_inline_mapping(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains(": ") && !trimmed.starts_with('-') && !trimmed.starts_with('#')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_length_detection() {
        let content = "short\nthis is a very long line that exceeds the maximum length limit we have set\nok";
        let ctx = RuleContext::new(content);
        let rule = LineLength::default();
        let diagnostics = rule.check(&ctx, None);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_allow_non_breakable_words() {
        let content = "url: https://very-long-url.example.com/with/a/very/long/path/that/exceeds/limit";
        let ctx = RuleContext::new(content);
        let rule = LineLength {
            allow_non_breakable_words: true,
            ..Default::default()
        };
        let diagnostics = rule.check(&ctx, None);
        assert!(diagnostics.is_empty());
    }
}
