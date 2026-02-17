use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct OctalValues {
    pub forbid_implicit_octal: bool,
    pub forbid_explicit_octal: bool,
}

impl Default for OctalValues {
    fn default() -> Self {
        Self {
            forbid_implicit_octal: true,
            forbid_explicit_octal: true,
        }
    }
}

impl Rule for OctalValues {
    fn name(&self) -> &'static str {
        "octal-values"
    }

    fn description(&self) -> &'static str {
        "Forbid specific octal values"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let forbid_implicit_octal = config
            .and_then(|c| c.get_option("forbid-implicit-octal"))
            .unwrap_or(self.forbid_implicit_octal);
        let forbid_explicit_octal = config
            .and_then(|c| c.get_option("forbid-explicit-octal"))
            .unwrap_or(self.forbid_explicit_octal);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            for part in trimmed.split_whitespace() {
                if forbid_implicit_octal
                    && part.starts_with('0')
                    && part.len() > 1
                    && part.chars().skip(1).all(|c| c.is_ascii_digit())
                    && !part.contains('.')
                {
                    // Exclude floats like 0.123
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "implicit octal value is forbidden",
                        crate::diagnostic::Location::new(
                            line_num,
                            line.find(part).unwrap_or(0) + 1,
                        ),
                    ));
                }
                if forbid_explicit_octal
                    && (part.starts_with("0o") || part.starts_with("0O"))
                    && part.len() > 2
                    && part.chars().skip(2).all(|c| c.is_digit(8))
                {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "explicit octal value is forbidden",
                        crate::diagnostic::Location::new(
                            line_num,
                            line.find(part).unwrap_or(0) + 1,
                        ),
                    ));
                }
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
