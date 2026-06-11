//! `octal-values`: forbid implicit and explicit octal values.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "octal-values";

#[derive(Debug, Clone)]
pub struct Conf {
    pub forbid_implicit_octal: bool,
    pub forbid_explicit_octal: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            forbid_implicit_octal: true,
            forbid_explicit_octal: true,
        }
    }
}

fn is_octal_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| ('0'..='7').contains(&c))
}

pub fn check(conf: &Conf, elem: &TokenElem, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    if matches!(
        elem.prev.as_ref().map(|t| &t.kind),
        Some(TokenKind::Tag { .. })
    ) {
        return;
    }

    if let TokenKind::Scalar { value, style, .. } = &token.kind
        && style.is_none()
    {
        if conf.forbid_implicit_octal {
            let val = value.as_str();
            if val.chars().all(|c| c.is_ascii_digit())
                && !val.is_empty()
                && val.chars().count() > 1
                && val.starts_with('0')
                && is_octal_digits(&val[1..])
            {
                problems.push(problem_at(
                    token.start_mark.line + 1,
                    token.end_mark.column + 1,
                    format!("forbidden implicit octal value \"{val}\""),
                ));
            }
        }

        if conf.forbid_explicit_octal {
            let val = value.as_str();
            if val.len() > 2 && val.starts_with("0o") && is_octal_digits(&val[2..]) {
                problems.push(problem_at(
                    token.start_mark.line + 1,
                    token.end_mark.column + 1,
                    format!("forbidden explicit octal value \"{val}\""),
                ));
            }
        }
    }
}
