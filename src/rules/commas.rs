//! `commas`: control the number of spaces before and after commas.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::{problem_at, spaces_after, spaces_before};

pub const ID: &str = "commas";

#[derive(Debug, Clone)]
pub struct Conf {
    pub max_spaces_before: i64,
    pub min_spaces_after: i64,
    pub max_spaces_after: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max_spaces_before: 0,
            min_spaces_after: 1,
            max_spaces_after: 1,
        }
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();
    let next = elem.next.as_ref();

    if matches!(token.kind, TokenKind::FlowEntry) {
        if let Some(prev_token) = prev {
            if conf.max_spaces_before != -1 && prev_token.end_mark.line < token.start_mark.line {
                problems.push(problem_at(
                    token.start_mark.line + 1,
                    std::cmp::max(1, token.start_mark.column),
                    "too many spaces before comma".to_string(),
                ));
            } else if let Some(problem) = spaces_before(
                token,
                prev,
                buffer,
                -1,
                conf.max_spaces_before,
                "",
                "too many spaces before comma",
            ) {
                problems.push(problem);
            }
        } else if let Some(problem) = spaces_before(
            token,
            prev,
            buffer,
            -1,
            conf.max_spaces_before,
            "",
            "too many spaces before comma",
        ) {
            problems.push(problem);
        }

        if let Some(problem) = spaces_after(
            token,
            next,
            conf.min_spaces_after,
            conf.max_spaces_after,
            "too few spaces after comma",
            "too many spaces after comma",
        ) {
            problems.push(problem);
        }
    }
}
