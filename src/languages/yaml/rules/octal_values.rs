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

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
