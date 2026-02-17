use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

#[derive(Default)]
pub struct FloatValues {
    pub forbid_inf: bool,
    pub forbid_nan: bool,
    pub forbid_scientific_notation: bool,
    pub require_numeral_before_decimal: bool,
}

impl Rule for FloatValues {
    fn name(&self) -> &'static str {
        "float-values"
    }

    fn description(&self) -> &'static str {
        "Forbid specific float values"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let forbid_inf = config
            .and_then(|c| c.get_option("forbid-inf"))
            .unwrap_or(self.forbid_inf);
        let forbid_nan = config
            .and_then(|c| c.get_option("forbid-nan"))
            .unwrap_or(self.forbid_nan);
        let forbid_scientific_notation = config
            .and_then(|c| c.get_option("forbid-scientific-notation"))
            .unwrap_or(self.forbid_scientific_notation);
        let require_numeral_before_decimal = config
            .and_then(|c| c.get_option("require-numeral-before-decimal"))
            .unwrap_or(self.require_numeral_before_decimal);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let parts: Vec<&str> = line.split_whitespace().collect();
            for part in parts {
                if forbid_inf && (part == ".inf" || part == "-.inf" || part == "+.inf") {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "forbidden value 'inf'",
                        crate::diagnostic::Location::new(
                            line_num,
                            line.find(part).unwrap_or(0) + 1,
                        ),
                    ));
                }
                if forbid_nan && part == ".nan" {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "forbidden value 'nan'",
                        crate::diagnostic::Location::new(
                            line_num,
                            line.find(part).unwrap_or(0) + 1,
                        ),
                    ));
                }
                if forbid_scientific_notation
                    && part.contains('e')
                    && part.chars().any(|c| c.is_ascii_digit())
                {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "scientific notation is forbidden",
                        crate::diagnostic::Location::new(
                            line_num,
                            line.find(part).unwrap_or(0) + 1,
                        ),
                    ));
                }
                if require_numeral_before_decimal
                    && part.starts_with('.')
                    && part.chars().nth(1).is_some_and(|c| c.is_ascii_digit())
                {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "missing numeral before decimal",
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
