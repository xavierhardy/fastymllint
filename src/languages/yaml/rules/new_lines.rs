use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewLineType {
    Unix,
    Dos,
    Platform,
}

impl Default for NewLineType {
    fn default() -> Self {
        NewLineType::Unix
    }
}

pub struct NewLines {
    pub type_: NewLineType,
}

impl Default for NewLines {
    fn default() -> Self {
        Self {
            type_: NewLineType::default(),
        }
    }
}

impl Rule for NewLines {
    fn name(&self) -> &'static str {
        "new-lines"
    }

    fn description(&self) -> &'static str {
        "Enforce new line type"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        true
    }
}
