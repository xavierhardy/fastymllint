//! `anchors`: report duplicated anchors and aliases referencing undeclared
//! anchors.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "anchors";

#[derive(Debug, Clone)]
pub struct Conf {
    pub forbid_undeclared_aliases: bool,
    pub forbid_duplicated_anchors: bool,
    pub forbid_unused_anchors: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            forbid_undeclared_aliases: true,
            forbid_duplicated_anchors: false,
            forbid_unused_anchors: false,
        }
    }
}

#[derive(Debug, Clone)]
struct AnchorInfo {
    line: usize,
    column: usize,
    used: bool,
}

#[derive(Debug, Default)]
pub struct Context {
    // Insertion-ordered so unused anchors are reported in declaration order.
    anchors: Vec<(String, AnchorInfo)>,
    initialized: bool,
}

impl Context {
    fn contains(&self, name: &str) -> bool {
        self.anchors.iter().any(|(n, _)| n == name)
    }
}

pub fn check(conf: &Conf, elem: &TokenElem, ctx: &mut Context, problems: &mut Vec<LintProblem>) {
    let token = &elem.curr;
    let next = elem.next.as_ref();
    let any_enabled = conf.forbid_undeclared_aliases
        || conf.forbid_duplicated_anchors
        || conf.forbid_unused_anchors;

    if any_enabled
        && matches!(
            token.kind,
            TokenKind::StreamStart | TokenKind::DocumentStart | TokenKind::DocumentEnd
        )
    {
        ctx.anchors.clear();
        ctx.initialized = true;
    }

    if conf.forbid_undeclared_aliases
        && let TokenKind::Alias(value) = &token.kind
        && !ctx.contains(value)
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("found undeclared alias \"{value}\""),
        ));
    }

    if conf.forbid_duplicated_anchors
        && let TokenKind::Anchor(value) = &token.kind
        && ctx.contains(value)
    {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            format!("found duplicated anchor \"{value}\""),
        ));
    }

    if conf.forbid_unused_anchors {
        // Unused anchors can only be detected at the end of Document.
        if matches!(
            next.map(|t| &t.kind),
            Some(TokenKind::StreamEnd)
                | Some(TokenKind::DocumentStart)
                | Some(TokenKind::DocumentEnd)
        ) {
            for (anchor, info) in &ctx.anchors {
                if !info.used {
                    problems.push(problem_at(
                        info.line + 1,
                        info.column + 1,
                        format!("found unused anchor \"{anchor}\""),
                    ));
                }
            }
        } else if let TokenKind::Alias(value) = &token.kind
            && let Some((_, info)) = ctx.anchors.iter_mut().find(|(n, _)| n == value)
        {
            info.used = true;
        }
    }

    if any_enabled && let TokenKind::Anchor(value) = &token.kind {
        let info = AnchorInfo {
            line: token.start_mark.line,
            column: token.start_mark.column,
            used: false,
        };
        if let Some(entry) = ctx.anchors.iter_mut().find(|(n, _)| n == value) {
            entry.1 = info;
        } else {
            ctx.anchors.push((value.clone(), info));
        }
    }
}
