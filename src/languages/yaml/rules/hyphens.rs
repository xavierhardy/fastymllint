use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct Hyphens {
    pub max_spaces_after: usize,
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

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
