use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    Single,
    Double,
    Consistent,
    Any,
}

impl Default for QuoteType {
    fn default() -> Self {
        QuoteType::Any
    }
}

pub struct QuotedStrings {
    pub quote_type: QuoteType,
    pub required: bool, // Simplified for skeleton, can effectively handle 'true', 'false', 'only-when-needed' logic
    pub extra_required: Vec<String>,
    pub extra_allowed: Vec<String>,
    pub allow_quoted_quotes: bool,
    pub check_keys: bool,
}

impl Default for QuotedStrings {
    fn default() -> Self {
        Self {
            quote_type: QuoteType::default(),
            required: true,
            extra_required: Vec::new(),
            extra_allowed: Vec::new(),
            allow_quoted_quotes: false,
            check_keys: false,
        }
    }
}

impl Rule for QuotedStrings {
    fn name(&self) -> &'static str {
        "quoted-strings"
    }

    fn description(&self) -> &'static str {
        "Enforce string quoting rules"
    }

    fn check(&self, _ctx: &RuleContext) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
