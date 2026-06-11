//! `key-duplicates`: prevent multiple identical keys in mappings.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "key-duplicates";

#[derive(Debug, Clone, Default)]
pub struct Conf {
    pub forbid_duplicated_merge_keys: bool,
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
            if let Some(TokenKind::Scalar { value, .. }) = next.map(|t| &t.kind) {
                // KeyTokens can be found inside flow sequences.
                if let Some(top) = ctx.stack.last_mut()
                    && top.r#type == ParentType::Map
                {
                    if top.keys.contains(value)
                            // `<<` is the merge key.
                            && (value != "<<" || conf.forbid_duplicated_merge_keys)
                    {
                        let next = next.unwrap();
                        problems.push(problem_at(
                            next.start_mark.line + 1,
                            next.start_mark.column + 1,
                            format!("duplication of key \"{value}\" in mapping"),
                        ));
                    } else {
                        top.keys.push(value.clone());
                    }
                }
            }
        }
        _ => {}
    }
}
