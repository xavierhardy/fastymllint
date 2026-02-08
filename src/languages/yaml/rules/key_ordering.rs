use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

#[derive(Default)]
pub struct KeyOrdering {
    pub ignored_keys: Vec<String>,
}

impl Rule for KeyOrdering {
    fn name(&self) -> &'static str {
        "key-ordering"
    }

    fn description(&self) -> &'static str {
        "Enforce alphabetical ordering of keys"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
