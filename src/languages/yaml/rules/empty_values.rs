use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct EmptyValues {
    pub forbid_in_block_mappings: bool,
    pub forbid_in_flow_mappings: bool,
    pub forbid_in_block_sequences: bool,
}

impl Default for EmptyValues {
    fn default() -> Self {
        Self {
            forbid_in_block_mappings: true,
            forbid_in_flow_mappings: true,
            forbid_in_block_sequences: true,
        }
    }
}

impl Rule for EmptyValues {
    fn name(&self) -> &'static str {
        "empty-values"
    }

    fn description(&self) -> &'static str {
        "Forbid empty values"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
