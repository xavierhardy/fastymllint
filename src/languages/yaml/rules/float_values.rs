use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct FloatValues {
    pub forbid_inf: bool,
    pub forbid_nan: bool,
    pub forbid_scientific_notation: bool,
    pub require_numeral_before_decimal: bool,
}

impl Default for FloatValues {
    fn default() -> Self {
        Self {
            forbid_inf: false,
            forbid_nan: false,
            forbid_scientific_notation: false,
            require_numeral_before_decimal: false,
        }
    }
}

impl Rule for FloatValues {
    fn name(&self) -> &'static str {
        "float-values"
    }

    fn description(&self) -> &'static str {
        "Forbid specific float values"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
