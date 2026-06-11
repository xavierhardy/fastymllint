//! `empty-values`: forbid nodes with empty content.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "empty-values";

#[derive(Debug, Clone)]
pub struct Conf {
    pub forbid_in_block_mappings: bool,
    pub forbid_in_flow_mappings: bool,
    pub forbid_in_block_sequences: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            forbid_in_block_mappings: true,
            forbid_in_flow_mappings: true,
            forbid_in_block_sequences: true,
        }
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let next = elem.next.as_ref().map(|t| &t.kind);

    if conf.forbid_in_block_mappings
        && matches!(token.kind, TokenKind::Value)
        && matches!(next, Some(TokenKind::Key) | Some(TokenKind::BlockEnd))
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.end_mark.column + 1,
            "empty value in block mapping".to_string(),
        ));
    }

    if conf.forbid_in_flow_mappings
        && matches!(token.kind, TokenKind::Value)
        && matches!(
            next,
            Some(TokenKind::FlowEntry) | Some(TokenKind::FlowMappingEnd)
        )
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.end_mark.column + 1,
            "empty value in flow mapping".to_string(),
        ));
    }

    if conf.forbid_in_block_sequences
        && matches!(token.kind, TokenKind::BlockEntry)
        && matches!(
            next,
            Some(TokenKind::Key) | Some(TokenKind::BlockEnd) | Some(TokenKind::BlockEntry)
        )
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.end_mark.column + 1,
            "empty value in block sequence".to_string(),
        ));
    }
}
