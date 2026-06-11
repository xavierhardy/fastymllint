//! `document-start`: require or forbid the document start marker (`---`).

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "document-start";

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
        let prev_matches = matches!(
            prev.map(|t| &t.kind),
            Some(TokenKind::StreamStart)
                | Some(TokenKind::DocumentEnd)
                | Some(TokenKind::Directive { .. })
        );
        let token_excluded = matches!(
            token.kind,
            TokenKind::DocumentStart | TokenKind::Directive { .. } | TokenKind::StreamEnd
        );
        if prev_matches && !token_excluded {
            problems.push(problem_at(
                token.start_mark.line + 1,
                1,
                "missing document start \"---\"".to_string(),
            ));
        }
    } else if matches!(token.kind, TokenKind::DocumentStart) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            "found forbidden document start \"---\"".to_string(),
        ));
    }
}
