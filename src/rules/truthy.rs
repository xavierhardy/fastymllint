//! `truthy`: forbid non-explicitly typed truthy values other than allowed
//! ones.

use std::collections::HashSet;

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::{DirectiveValue, TokenKind};
use crate::rules::common::problem_at;

pub const ID: &str = "truthy";

pub const TRUTHY_1_1: [&str; 18] = [
    "YES", "Yes", "yes", "NO", "No", "no", "TRUE", "True", "true", "FALSE", "False", "false", "ON",
    "On", "on", "OFF", "Off", "off",
];
pub const TRUTHY_1_2: [&str; 6] = ["TRUE", "True", "true", "FALSE", "False", "false"];

#[derive(Debug, Clone)]
pub struct Conf {
    pub allowed_values: Vec<String>,
    pub check_keys: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            allowed_values: vec!["true".to_string(), "false".to_string()],
            check_keys: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct Context {
    yaml_spec_version: Option<(u64, u64)>,
    bad_truthy_values: Option<HashSet<String>>,
}

pub fn check(conf: &Conf, elem: &TokenElem, ctx: &mut Context, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();

    if let TokenKind::Directive { name, value } = &token.kind {
        if name == "YAML"
            && let DirectiveValue::Yaml(major, minor) = value
        {
            ctx.yaml_spec_version = Some((*major, *minor));
        }
    } else if matches!(token.kind, TokenKind::DocumentEnd) {
        ctx.yaml_spec_version = None;
        ctx.bad_truthy_values = None;
    }

    if matches!(prev.map(|t| &t.kind), Some(TokenKind::Tag { .. })) {
        return;
    }

    if !conf.check_keys
        && matches!(prev.map(|t| &t.kind), Some(TokenKind::Key))
        && matches!(token.kind, TokenKind::Scalar { .. })
    {
        return;
    }

    if let TokenKind::Scalar {
        value, style: None, ..
    } = &token.kind
    {
        if ctx.bad_truthy_values.is_none() {
            let base: &[&str] = if ctx.yaml_spec_version == Some((1, 2)) {
                &TRUTHY_1_2
            } else {
                &TRUTHY_1_1
            };
            let mut set: HashSet<String> = base.iter().map(|s| s.to_string()).collect();
            for allowed in &conf.allowed_values {
                set.remove(allowed);
            }
            ctx.bad_truthy_values = Some(set);
        }

        if ctx
            .bad_truthy_values
            .as_ref()
            .is_some_and(|set| set.contains(value))
        {
            let mut allowed = conf.allowed_values.clone();
            allowed.sort();
            problems.push(problem_at(
                token.start_mark.line + 1,
                token.start_mark.column + 1,
                format!("truthy value should be one of [{}]", allowed.join(", ")),
            ));
        }
    }
}
