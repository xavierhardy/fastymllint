//! `float-values`: restrict permitted float values.

use std::sync::LazyLock;

use regex::Regex;

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "float-values";

#[derive(Debug, Clone, Default)]
pub struct Conf {
    pub require_numeral_before_decimal: bool,
    pub forbid_scientific_notation: bool,
    pub forbid_nan: bool,
    pub forbid_inf: bool,
}

static IS_NUMERAL_BEFORE_DECIMAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[-+]?(\.[0-9]+)([eE][-+]?[0-9]+)?$").unwrap());
static IS_SCIENTIFIC_NOTATION: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[-+]?(\.[0-9]+|[0-9]+(\.[0-9]*)?)([eE][-+]?[0-9]+)$").unwrap());
static IS_INF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[-+]?(\.inf|\.Inf|\.INF)$").unwrap());
static IS_NAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\.nan|\.NaN|\.NAN)$").unwrap());

pub fn check(conf: &Conf, elem: &TokenElem, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    if matches!(
        elem.prev.as_ref().map(|t| &t.kind),
        Some(TokenKind::Tag { .. })
    ) {
        return;
    }
    let TokenKind::Scalar { value, style, .. } = &token.kind else {
        return;
    };
    if style.is_some() {
        return;
    }

    if conf.forbid_nan && IS_NAN.is_match(value) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("forbidden not a number value \"{value}\""),
        ));
    }

    if conf.forbid_inf && IS_INF.is_match(value) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("forbidden infinite value \"{value}\""),
        ));
    }

    if conf.forbid_scientific_notation && IS_SCIENTIFIC_NOTATION.is_match(value) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("forbidden scientific notation \"{value}\""),
        ));
    }

    if conf.require_numeral_before_decimal && IS_NUMERAL_BEFORE_DECIMAL.is_match(value) {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("forbidden decimal missing 0 prefix \"{value}\""),
        ));
    }
}
