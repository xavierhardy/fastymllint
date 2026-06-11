//! `brackets`: control the use of flow sequences and spaces inside brackets.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::braces::Forbid;
use crate::rules::common::{problem_at, spaces_after, spaces_before};

pub const ID: &str = "brackets";

#[derive(Debug, Clone)]
pub struct Conf {
    pub forbid: Forbid,
    pub min_spaces_inside: i64,
    pub max_spaces_inside: i64,
    pub min_spaces_inside_empty: i64,
    pub max_spaces_inside_empty: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            forbid: Forbid::No,
            min_spaces_inside: 0,
            max_spaces_inside: 0,
            min_spaces_inside_empty: -1,
            max_spaces_inside_empty: -1,
        }
    }
}

// Parallel branches are kept separate on purpose: each one corresponds to
// a distinct check documented for this rule.
#[allow(clippy::if_same_then_else)]
pub fn check(conf: &Conf, elem: &TokenElem, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();
    let next = elem.next.as_ref();

    if conf.forbid == Forbid::Yes && matches!(token.kind, TokenKind::FlowSequenceStart) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.end_mark.column + 1,
            "forbidden flow sequence".to_string(),
        ));
    } else if conf.forbid == Forbid::NonEmpty
        && matches!(token.kind, TokenKind::FlowSequenceStart)
        && !matches!(next.map(|t| &t.kind), Some(TokenKind::FlowSequenceEnd))
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.end_mark.column + 1,
            "forbidden flow sequence".to_string(),
        ));
    } else if matches!(token.kind, TokenKind::FlowSequenceStart)
        && matches!(next.map(|t| &t.kind), Some(TokenKind::FlowSequenceEnd))
    {
        let min = if conf.min_spaces_inside_empty != -1 {
            conf.min_spaces_inside_empty
        } else {
            conf.min_spaces_inside
        };
        let max = if conf.max_spaces_inside_empty != -1 {
            conf.max_spaces_inside_empty
        } else {
            conf.max_spaces_inside
        };
        if let Some(problem) = spaces_after(
            token,
            next,
            min,
            max,
            "too few spaces inside empty brackets",
            "too many spaces inside empty brackets",
        ) {
            problems.push(problem);
        }
    } else if matches!(token.kind, TokenKind::FlowSequenceStart) {
        if let Some(problem) = spaces_after(
            token,
            next,
            conf.min_spaces_inside,
            conf.max_spaces_inside,
            "too few spaces inside brackets",
            "too many spaces inside brackets",
        ) {
            problems.push(problem);
        }
    } else if matches!(token.kind, TokenKind::FlowSequenceEnd)
        && !matches!(prev.map(|t| &t.kind), Some(TokenKind::FlowSequenceStart))
        && let Some(problem) = spaces_before(
            token,
            prev,
            buffer,
            conf.min_spaces_inside,
            conf.max_spaces_inside,
            "too few spaces inside brackets",
            "too many spaces inside brackets",
        )
    {
        problems.push(problem);
    }
}
