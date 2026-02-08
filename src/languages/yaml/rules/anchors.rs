use crate::diagnostic::Diagnostic;
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
            forbid_duplicated_anchors: false,
            forbid_unused_anchors: false,
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

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
