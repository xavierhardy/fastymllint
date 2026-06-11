//! `colons`: control the number of spaces before and after colons.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::{is_explicit_key, spaces_after, spaces_before};

pub const ID: &str = "colons";

#[derive(Debug, Clone)]
pub struct Conf {
    pub max_spaces_before: i64,
    pub max_spaces_after: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max_spaces_before: 0,
            max_spaces_after: 1,
        }
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();
    let next = elem.next.as_ref();

    if matches!(token.kind, TokenKind::Value)
        && !(matches!(prev.map(|t| &t.kind), Some(TokenKind::Alias(_)))
            && token.start_mark.pointer
                == prev.map(|t| t.end_mark.pointer).unwrap_or(usize::MAX) + 1)
    {
        if let Some(problem) = spaces_before(
            token,
            prev,
            buffer,
            -1,
            conf.max_spaces_before,
            "",
            "too many spaces before colon",
        ) {
            problems.push(problem);
        }

        if let Some(problem) = spaces_after(
            token,
            next,
            -1,
            conf.max_spaces_after,
            "",
            "too many spaces after colon",
        ) {
            problems.push(problem);
        }
    }

    if matches!(token.kind, TokenKind::Key)
        && is_explicit_key(token, buffer)
        && let Some(problem) = spaces_after(
            token,
            next,
            -1,
            conf.max_spaces_after,
            "",
            "too many spaces after question mark",
        )
    {
        problems.push(problem);
    }
}
