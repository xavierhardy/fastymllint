//! `document-end`: require or forbid the document end marker (`...`).

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "document-end";

#[derive(Debug, Clone)]
pub struct Conf {
    pub present: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self { present: true }
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();

    if conf.present {
        let is_stream_end = matches!(token.kind, TokenKind::StreamEnd);
        let is_start = matches!(token.kind, TokenKind::DocumentStart);
        let prev_is_end_or_stream_start = matches!(
            prev.map(|t| &t.kind),
            Some(TokenKind::DocumentEnd) | Some(TokenKind::StreamStart)
        );
        let prev_is_directive = matches!(prev.map(|t| &t.kind), Some(TokenKind::Directive { .. }));

        if is_stream_end && !prev_is_end_or_stream_start {
            problems.push(problem_at(
                token.start_mark.line,
                1,
                "missing document end \"...\"".to_string(),
            ));
        } else if is_start && !(prev_is_end_or_stream_start || prev_is_directive) {
            problems.push(problem_at(
                token.start_mark.line + 1,
                1,
                "missing document end \"...\"".to_string(),
            ));
        }
    } else if matches!(token.kind, TokenKind::DocumentEnd) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            "found forbidden document end \"...\"".to_string(),
        ));
    }
}
