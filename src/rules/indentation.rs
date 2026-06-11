//! `indentation`: control indentation.
//! machine.

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::tokens::{Token, TokenKind};
use crate::rules::common::{get_real_end_line, is_explicit_key, problem_at};

pub const ID: &str = "indentation";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Spaces {
    Consistent,
    Fixed(i64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndentSequences {
    True,
    False,
    Whatever,
    Consistent,
}

#[derive(Debug, Clone)]
pub struct Conf {
    pub spaces: Spaces,
    pub indent_sequences: IndentSequences,
    pub check_multi_line_strings: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            spaces: Spaces::Consistent,
            indent_sequences: IndentSequences::True,
            check_multi_line_strings: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParentType {
    Root,
    BMap,
    FMap,
    BSeq,
    FSeq,
    BEnt,
    Key,
    Val,
}

#[derive(Debug)]
struct Parent {
    r#type: ParentType,
    indent: i64,
    line_indent: Option<i64>,
    explicit_key: bool,
    implicit_block_seq: bool,
}

impl Parent {
    fn new(r#type: ParentType, indent: i64) -> Self {
        Self {
            r#type,
            indent,
            line_indent: None,
            explicit_key: false,
            implicit_block_seq: false,
        }
    }
}

#[derive(Debug)]
pub struct Context {
    stack: Vec<Parent>,
    cur_line: i64,
    cur_line_indent: i64,
    spaces: Spaces,
    indent_sequences: IndentSequences,
    initialized: bool,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            cur_line: -1,
            cur_line_indent: 0,
            spaces: Spaces::Consistent,
            indent_sequences: IndentSequences::True,
            initialized: false,
        }
    }
}

fn detect_indent(spaces: &mut Spaces, base_indent: i64, next_column: i64) -> i64 {
    if let Spaces::Consistent = spaces {
        *spaces = Spaces::Fixed(next_column - base_indent);
    }
    match spaces {
        Spaces::Fixed(n) => base_indent + *n,
        Spaces::Consistent => unreachable!(),
    }
}

fn check_scalar_indentation(
    token: &Token,
    buffer: &[char],
    ctx: &mut Context,
    problems: &mut Vec<LintProblem>,
) {
    if token.start_mark.line == token.end_mark.line {
        return;
    }

    let token_style = token.scalar_style();
    let token_plain = token.scalar_is_plain();

    let compute_expected_indent = |ctx: &mut Context, found_indent: i64| -> Option<i64> {
        if token_plain {
            return Some(token.start_mark.column as i64);
        } else if matches!(token_style, Some('"') | Some('\'')) {
            return Some(token.start_mark.column as i64 + 1);
        } else if matches!(token_style, Some('>') | Some('|')) {
            let top_type = ctx.stack.last().map(|p| p.r#type);
            match top_type {
                Some(ParentType::BEnt) | Some(ParentType::Key) => {
                    // - >
                    //     multi
                    //     line
                    let base = token.start_mark.column as i64;
                    if let Spaces::Consistent = ctx.spaces {
                        ctx.spaces = Spaces::Fixed(found_indent - base);
                    }
                    if let Spaces::Fixed(n) = ctx.spaces {
                        return Some(base + n);
                    }
                }
                Some(ParentType::Val) => {
                    let base;
                    if token.start_mark.line as i64 + 1 > ctx.cur_line {
                        // - key:
                        //     >
                        //       multi
                        //       line
                        base = ctx.stack.last().unwrap().indent;
                    } else if ctx.stack.len() >= 2 && ctx.stack[ctx.stack.len() - 2].explicit_key {
                        // - ? key
                        //   : >
                        //       multi-line
                        //       value
                        base = token.start_mark.column as i64;
                    } else {
                        // - key: >
                        //     multi
                        //     line
                        base = ctx.stack[ctx.stack.len() - 2].indent;
                    }
                    if let Spaces::Consistent = ctx.spaces {
                        ctx.spaces = Spaces::Fixed(found_indent - base);
                    }
                    if let Spaces::Fixed(n) = ctx.spaces {
                        return Some(base + n);
                    }
                }
                _ => {
                    let base = ctx.stack.last().map(|p| p.indent).unwrap_or(0);
                    if let Spaces::Consistent = ctx.spaces {
                        ctx.spaces = Spaces::Fixed(found_indent - base);
                    }
                    if let Spaces::Fixed(n) = ctx.spaces {
                        return Some(base + n);
                    }
                }
            }
        }
        None
    };

    let mut expected_indent: Option<i64> = None;

    let mut line_no = token.start_mark.line + 1;

    let mut line_start = token.start_mark.pointer;
    loop {
        // find('\n', line_start, end_mark.pointer - 1) + 1
        let search_end = token.end_mark.pointer.saturating_sub(1).min(buffer.len());
        if line_start >= search_end {
            break;
        }
        match buffer[line_start..search_end]
            .iter()
            .position(|&c| c == '\n')
        {
            Some(pos) => line_start = line_start + pos + 1,
            None => break,
        }
        line_no += 1;

        let mut indent: i64 = 0;
        while buffer.get(line_start + indent as usize).copied() == Some(' ') {
            indent += 1;
        }
        if buffer.get(line_start + indent as usize).copied() == Some('\n') {
            continue;
        }

        if expected_indent.is_none() {
            expected_indent = compute_expected_indent(ctx, indent);
        }

        if let Some(expected) = expected_indent
            && expected != indent
        {
            problems.push(problem_at(
                line_no,
                (indent + 1) as usize,
                format!("wrong indentation: expected {expected} but found {indent}"),
            ));
        }
    }
}

/// Returns `Err(())` when the token stream does not match the structure
/// this rule expects (reported as "cannot infer indentation").
// Parallel branches are kept separate on purpose: each one corresponds to
// a distinct check documented for this rule.
#[allow(clippy::if_same_then_else)]
fn check_inner(
    conf: &Conf,
    elem: &TokenElem,
    buffer: &[char],
    ctx: &mut Context,
    problems: &mut Vec<LintProblem>,
) -> Result<(), ()> {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();
    let next = elem.next.as_ref();
    let nextnext = elem.nextnext.as_ref();

    if !ctx.initialized {
        ctx.stack = vec![Parent::new(ParentType::Root, 0)];
        ctx.cur_line = -1;
        ctx.spaces = conf.spaces;
        ctx.indent_sequences = conf.indent_sequences;
        ctx.initialized = true;
    }

    // Step 1: Lint

    let is_visible = !matches!(
        token.kind,
        TokenKind::StreamStart | TokenKind::StreamEnd | TokenKind::BlockEnd
    ) && !matches!(&token.kind, TokenKind::Scalar { value, .. } if value.is_empty());
    let first_in_line = is_visible && token.start_mark.line as i64 + 1 > ctx.cur_line;

    let mut found_indentation: i64 = 0;
    if first_in_line {
        found_indentation = token.start_mark.column as i64;
        let mut expected = ctx.stack.last().map(|p| p.indent).unwrap_or(0);

        if matches!(
            token.kind,
            TokenKind::FlowMappingEnd | TokenKind::FlowSequenceEnd
        ) {
            expected = ctx
                .stack
                .last()
                .and_then(|p| p.line_indent)
                .unwrap_or(expected);
        } else if ctx
            .stack
            .last()
            .map(|p| p.r#type == ParentType::Key && p.explicit_key)
            .unwrap_or(false)
            && !matches!(token.kind, TokenKind::Value)
        {
            expected = detect_indent(&mut ctx.spaces, expected, token.start_mark.column as i64);
        }

        if found_indentation != expected {
            let message = if expected < 0 {
                format!(
                    "wrong indentation: expected at least {}",
                    found_indentation + 1
                )
            } else {
                format!("wrong indentation: expected {expected} but found {found_indentation}")
            };
            problems.push(problem_at(
                token.start_mark.line + 1,
                (found_indentation + 1) as usize,
                message,
            ));
        }
    }

    if matches!(token.kind, TokenKind::Scalar { .. }) && conf.check_multi_line_strings {
        check_scalar_indentation(token, buffer, ctx, problems);
    }

    // Step 2.a:

    if is_visible {
        ctx.cur_line = get_real_end_line(token, buffer) as i64;
        if first_in_line {
            ctx.cur_line_indent = found_indentation;
        }
    }

    // Step 2.b: Update state

    match &token.kind {
        TokenKind::BlockMappingStart => {
            //   - a: 1
            let next_t = next.ok_or(())?;
            if !matches!(next_t.kind, TokenKind::Key) {
                return Err(());
            }
            if next_t.start_mark.line != token.start_mark.line {
                return Err(());
            }

            let indent = token.start_mark.column as i64;
            ctx.stack.push(Parent::new(ParentType::BMap, indent));
        }
        TokenKind::FlowMappingStart => {
            let next_t = next.ok_or(())?;
            let indent = if next_t.start_mark.line == token.start_mark.line {
                //   - {a: 1, b: 2}
                next_t.start_mark.column as i64
            } else {
                //   - {
                //     a: 1, b: 2
                //   }
                detect_indent(
                    &mut ctx.spaces,
                    ctx.cur_line_indent,
                    next_t.start_mark.column as i64,
                )
            };
            let mut parent = Parent::new(ParentType::FMap, indent);
            parent.line_indent = Some(ctx.cur_line_indent);
            ctx.stack.push(parent);
        }
        TokenKind::BlockSequenceStart => {
            //   - - a
            //     - b
            let next_t = next.ok_or(())?;
            if !matches!(next_t.kind, TokenKind::BlockEntry) {
                return Err(());
            }
            if next_t.start_mark.line != token.start_mark.line {
                return Err(());
            }

            let indent = token.start_mark.column as i64;
            ctx.stack.push(Parent::new(ParentType::BSeq, indent));
        }
        TokenKind::BlockEntry
            if !matches!(
                next.map(|t| &t.kind),
                Some(TokenKind::BlockEntry) | Some(TokenKind::BlockEnd)
            ) =>
        {
            // It looks like pyyaml doesn't issue BlockSequenceStartTokens
            // when the list is not indented. Compensate for that.
            if ctx.stack.last().map(|p| p.r#type) != Some(ParentType::BSeq) {
                let mut parent = Parent::new(ParentType::BSeq, token.start_mark.column as i64);
                parent.implicit_block_seq = true;
                ctx.stack.push(parent);
            }

            let next_t = next.ok_or(())?;
            let indent = if next_t.start_mark.line == token.end_mark.line {
                //   - item 1
                next_t.start_mark.column as i64
            } else if next_t.start_mark.column == token.start_mark.column {
                //   -
                //   key: value
                next_t.start_mark.column as i64
            } else {
                //   -
                //     item 1
                detect_indent(
                    &mut ctx.spaces,
                    token.start_mark.column as i64,
                    next_t.start_mark.column as i64,
                )
            };
            ctx.stack.push(Parent::new(ParentType::BEnt, indent));
        }
        TokenKind::FlowSequenceStart => {
            let next_t = next.ok_or(())?;
            let indent = if next_t.start_mark.line == token.start_mark.line {
                //   - [a, b]
                next_t.start_mark.column as i64
            } else {
                detect_indent(
                    &mut ctx.spaces,
                    ctx.cur_line_indent,
                    next_t.start_mark.column as i64,
                )
            };
            let mut parent = Parent::new(ParentType::FSeq, indent);
            parent.line_indent = Some(ctx.cur_line_indent);
            ctx.stack.push(parent);
        }
        TokenKind::Key => {
            let indent = ctx.stack.last().map(|p| p.indent).unwrap_or(0);
            let mut parent = Parent::new(ParentType::Key, indent);
            parent.explicit_key = is_explicit_key(token, buffer);
            ctx.stack.push(parent);
        }
        TokenKind::Value => {
            if ctx.stack.last().map(|p| p.r#type) != Some(ParentType::Key) {
                return Err(());
            }

            // Special cases:
            //     key: &anchor
            //       value
            // and:
            //     key: !!tag
            //       value
            let mut effective_next = next;
            if matches!(
                next.map(|t| &t.kind),
                Some(TokenKind::Anchor(_)) | Some(TokenKind::Tag { .. })
            ) && let (Some(next_t), Some(prev_t), Some(nextnext_t)) = (next, prev, nextnext)
                && next_t.start_mark.line == prev_t.start_mark.line
                && next_t.start_mark.line < nextnext_t.start_mark.line
            {
                effective_next = nextnext;
            }
            let next_t = effective_next;

            // Only if value is not empty.
            if !matches!(
                next_t.map(|t| &t.kind),
                Some(TokenKind::BlockEnd)
                    | Some(TokenKind::FlowMappingEnd)
                    | Some(TokenKind::FlowSequenceEnd)
                    | Some(TokenKind::Key)
                    | None
            ) {
                let next_t = next_t.unwrap();
                let top_indent = ctx.stack.last().unwrap().indent;
                let top_explicit = ctx.stack.last().unwrap().explicit_key;
                let indent;
                if top_explicit {
                    //   ? k
                    //   : value
                    indent =
                        detect_indent(&mut ctx.spaces, top_indent, next_t.start_mark.column as i64);
                } else if prev
                    .map(|p| next_t.start_mark.line == p.start_mark.line)
                    .unwrap_or(false)
                {
                    //   k: value
                    indent = next_t.start_mark.column as i64;
                } else if matches!(
                    next_t.kind,
                    TokenKind::BlockSequenceStart | TokenKind::BlockEntry
                ) {
                    // Sometimes BlockSequenceStartTokens are not issued.
                    if ctx.indent_sequences == IndentSequences::False {
                        indent = top_indent;
                    } else if ctx.indent_sequences == IndentSequences::True {
                        if ctx.spaces == Spaces::Consistent
                            && next_t.start_mark.column as i64 - top_indent == 0
                        {
                            // The block sequence item is not indented (while
                            // it should be), but we don't know yet the
                            // indentation it should have. Choose unknown (-1).
                            indent = -1;
                        } else {
                            indent = detect_indent(
                                &mut ctx.spaces,
                                top_indent,
                                next_t.start_mark.column as i64,
                            );
                        }
                    } else {
                        // 'whatever' or 'consistent'
                        if next_t.start_mark.column as i64 == top_indent {
                            //   key:
                            //   - e1
                            //   - e2
                            if ctx.indent_sequences == IndentSequences::Consistent {
                                ctx.indent_sequences = IndentSequences::False;
                            }
                            indent = top_indent;
                        } else {
                            if ctx.indent_sequences == IndentSequences::Consistent {
                                ctx.indent_sequences = IndentSequences::True;
                            }
                            //   key:
                            //     - e1
                            //     - e2
                            indent = detect_indent(
                                &mut ctx.spaces,
                                top_indent,
                                next_t.start_mark.column as i64,
                            );
                        }
                    }
                } else {
                    //   k:
                    //     value
                    indent =
                        detect_indent(&mut ctx.spaces, top_indent, next_t.start_mark.column as i64);
                }

                ctx.stack.push(Parent::new(ParentType::Val, indent));
            }
        }
        _ => {}
    }

    let mut consumed_current_token = false;
    while let Some(top) = ctx.stack.last() {
        let top_type = top.r#type;

        if top_type == ParentType::FSeq
            && matches!(token.kind, TokenKind::FlowSequenceEnd)
            && !consumed_current_token
        {
            ctx.stack.pop();
            consumed_current_token = true;
        } else if top_type == ParentType::FMap
            && matches!(token.kind, TokenKind::FlowMappingEnd)
            && !consumed_current_token
        {
            ctx.stack.pop();
            consumed_current_token = true;
        } else if matches!(top_type, ParentType::BMap | ParentType::BSeq)
            && matches!(token.kind, TokenKind::BlockEnd)
            && !ctx.stack.last().unwrap().implicit_block_seq
            && !consumed_current_token
        {
            ctx.stack.pop();
            consumed_current_token = true;
        } else if top_type == ParentType::BEnt
            && !matches!(token.kind, TokenKind::BlockEntry)
            && ctx.stack.len() >= 2
            && ctx.stack[ctx.stack.len() - 2].implicit_block_seq
            && !matches!(token.kind, TokenKind::Anchor(_) | TokenKind::Tag { .. })
            && !matches!(next.map(|t| &t.kind), Some(TokenKind::BlockEntry))
        {
            ctx.stack.pop();
            ctx.stack.pop();
        } else if top_type == ParentType::BEnt
            && matches!(
                next.map(|t| &t.kind),
                Some(TokenKind::BlockEntry) | Some(TokenKind::BlockEnd)
            )
        {
            ctx.stack.pop();
        } else if top_type == ParentType::Val
            && !matches!(token.kind, TokenKind::Value)
            && !matches!(token.kind, TokenKind::Anchor(_) | TokenKind::Tag { .. })
        {
            if ctx.stack.len() < 2 || ctx.stack[ctx.stack.len() - 2].r#type != ParentType::Key {
                return Err(());
            }
            ctx.stack.pop();
            ctx.stack.pop();
        } else if top_type == ParentType::Key
            && matches!(
                next.map(|t| &t.kind),
                Some(TokenKind::BlockEnd)
                    | Some(TokenKind::FlowMappingEnd)
                    | Some(TokenKind::FlowSequenceEnd)
                    | Some(TokenKind::Key)
            )
        {
            // A key without a value: it's part of a set.
            ctx.stack.pop();
        } else {
            break;
        }
    }

    Ok(())
}

pub fn check(
    conf: &Conf,
    elem: &TokenElem,
    buffer: &[char],
    ctx: &mut Context,
    problems: &mut Vec<LintProblem>,
) {
    if check_inner(conf, elem, buffer, ctx, problems).is_err() {
        problems.push(problem_at(
            elem.curr.start_mark.line + 1,
            elem.curr.start_mark.column + 1,
            "cannot infer indentation: unexpected token".to_string(),
        ));
    }
}
