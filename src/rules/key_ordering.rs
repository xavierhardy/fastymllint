//! `key-ordering`: enforce alphabetical ordering of keys in mappings.
//!
//! Ordering uses code-point comparison (equivalent to collation in the C
//! locale); the `locale` configuration option is not supported.

use regex::Regex;

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "key-ordering";

#[derive(Debug, Clone, Default)]
pub struct Conf {
    pub ignored_keys: Vec<Regex>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParentType {
    Map,
    Seq,
}

#[derive(Debug)]
struct Parent {
    r#type: ParentType,
    keys: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Context {
    stack: Vec<Parent>,
}

pub fn check(conf: &Conf, elem: &TokenElem, ctx: &mut Context, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let next = elem.next.as_ref();

    match &token.kind {
        TokenKind::BlockMappingStart | TokenKind::FlowMappingStart => {
            ctx.stack.push(Parent {
                r#type: ParentType::Map,
                keys: Vec::new(),
            });
        }
        TokenKind::BlockSequenceStart | TokenKind::FlowSequenceStart => {
            ctx.stack.push(Parent {
                r#type: ParentType::Seq,
                keys: Vec::new(),
            });
        }
        TokenKind::BlockEnd | TokenKind::FlowMappingEnd | TokenKind::FlowSequenceEnd => {
            ctx.stack.pop();
        }
        TokenKind::Key => {
            if let Some(TokenKind::Scalar { value, .. }) = next.map(|t| &t.kind)
                && let Some(top) = ctx.stack.last_mut()
                && top.r#type == ParentType::Map
                && !conf.ignored_keys.iter().any(|r| r.is_match(value))
            {
                if top.keys.iter().any(|key| value < key) {
                    let next = next.unwrap();
                    problems.push(problem_at(
                        next.start_mark.line + 1,
                        next.start_mark.column + 1,
                        format!("wrong ordering of key \"{value}\" in mapping"),
                    ));
                } else {
                    top.keys.push(value.clone());
                }
            }
        }
        _ => {}
    }
}
