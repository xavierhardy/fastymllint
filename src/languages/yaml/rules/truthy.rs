use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct Truthy {
    pub allowed_values: Vec<String>,
    pub check_keys: bool,
}

impl Default for Truthy {
    fn default() -> Self {
        Self {
            allowed_values: vec!["true".to_string(), "false".to_string()],
            check_keys: true,
        }
    }
}

impl Rule for Truthy {
    fn name(&self) -> &'static str {
        "truthy"
    }

    fn description(&self) -> &'static str {
        "Enforce truthy values"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
