//! `hyphens`: control the number of spaces after hyphens.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::spaces_after;

pub const ID: &str = "hyphens";

#[derive(Debug, Clone)]
pub struct Conf {
    pub max_spaces_after: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max_spaces_after: 1,
        }
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    if matches!(token.kind, TokenKind::BlockEntry)
        && let Some(problem) = spaces_after(
            token,
            elem.next.as_ref(),
            -1,
            conf.max_spaces_after,
            "",
            "too many spaces after hyphen",
        )
    {
        problems.push(problem);
    }
}
