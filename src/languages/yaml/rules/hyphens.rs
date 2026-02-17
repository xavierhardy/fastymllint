use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

pub struct Hyphens {
    pub max_spaces_after: i64,
}

impl Default for Hyphens {
    fn default() -> Self {
        Self {
            max_spaces_after: 1,
        }
    }
}

impl Rule for Hyphens {
    fn name(&self) -> &'static str {
        "hyphens"
    }

    fn description(&self) -> &'static str {
        "Enforce spacing after hyphens"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let max_spaces_after = config
            .and_then(|c| c.get_int("max-spaces-after"))
            .unwrap_or(self.max_spaces_after);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim_start();
            if trimmed.starts_with("- ") {
                let mut spaces = 0;
                for c in trimmed[1..].chars() {
                    if c == ' ' {
                        spaces += 1;
                    } else {
                        break;
                    }
                }
                if max_spaces_after >= 0 && (spaces as i64) > max_spaces_after {
                    let start_col = line.len() - trimmed.len() + 2;
                    let diag = Diagnostic::error(
                        self.name(),
                        "too many spaces after hyphen",
                        Location::new(line_num, start_col),
                    );
                    diagnostics.push(diag.with_fix(Fix::delete(
                        "remove extra spaces",
                        Location::new(line_num, start_col + max_spaces_after as usize),
                        Location::new(line_num, start_col + spaces),
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
