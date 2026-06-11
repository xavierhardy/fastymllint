//! `quoted-strings`: control whether string values are quoted.

use regex::Regex;

use crate::linter::{LintProblem, TokenElem};
use crate::pyyaml::resolver::{DEFAULT_SCALAR_TAG, resolve_scalar_tag};
use crate::pyyaml::scanner::Scanner;
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "quoted-strings";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QuoteType {
    Any,
    Single,
    Double,
    Consistent,
}

impl QuoteType {
    pub fn as_str(self) -> &'static str {
        match self {
            QuoteType::Any => "any",
            QuoteType::Single => "single",
            QuoteType::Double => "double",
            QuoteType::Consistent => "consistent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Required {
    True,
    False,
    OnlyWhenNeeded,
}

#[derive(Debug, Clone)]
pub struct Conf {
    pub quote_type: QuoteType,
    pub required: Required,
    pub extra_required: Vec<Regex>,
    pub extra_allowed: Vec<Regex>,
    pub allow_quoted_quotes: bool,
    pub check_keys: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            quote_type: QuoteType::Any,
            required: Required::True,
            extra_required: Vec::new(),
            extra_allowed: Vec::new(),
            allow_quoted_quotes: false,
            check_keys: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Context {
    flow_nest_count: i64,
    consistent_token_style: Option<Option<char>>,
}

fn quote_match(quote_type: QuoteType, token_style: Option<char>, ctx: &mut Context) -> bool {
    if quote_type == QuoteType::Consistent && token_style.is_some() {
        // The canonical token style in a document is assumed to be the first
        // one found.
        if ctx.consistent_token_style.is_none() {
            ctx.consistent_token_style = Some(token_style);
        }
        return ctx.consistent_token_style == Some(token_style);
    }

    matches!(
        (quote_type, token_style),
        (QuoteType::Any, _) | (QuoteType::Single, Some('\'')) | (QuoteType::Double, Some('"'))
    )
}

/// PyYAML reader printability check.
fn check_printable(s: &str) -> bool {
    s.chars().all(|c| {
        matches!(c, '\x09' | '\x0A' | '\x0D' | '\x20'..='\x7E' | '\u{85}')
            || ('\u{A0}'..='\u{D7FF}').contains(&c)
            || ('\u{E000}'..='\u{FFFD}').contains(&c)
            || c >= '\u{10000}'
    })
}

fn has_backslash_on_at_least_one_line_ending(elem: &TokenElem, buffer: &[char]) -> bool {
    let token = &elem.curr;
    if token.start_mark.line == token.end_mark.line {
        return false;
    }
    let start = token.start_mark.pointer + 1;
    let end = token.end_mark.pointer.saturating_sub(1);
    if start >= end {
        return false;
    }
    let slice = &buffer[start..end.min(buffer.len())];
    for i in 0..slice.len().saturating_sub(1) {
        if slice[i] == '\\' && slice[i + 1] == '\n' {
            return true;
        }
        if i + 2 < slice.len() && slice[i] == '\\' && slice[i + 1] == '\r' && slice[i + 2] == '\n' {
            return true;
        }
    }
    false
}

fn quotes_are_needed(elem: &TokenElem, buffer: &[char], is_inside_a_flow: bool) -> bool {
    let token = &elem.curr;
    let TokenKind::Scalar { value, style, .. } = &token.kind else {
        return true;
    };

    // Quotes needed on strings containing flow tokens.
    if is_inside_a_flow
        && value
            .chars()
            .any(|c| matches!(c, ',' | '[' | ']' | '{' | '}'))
    {
        return true;
    }

    if *style == Some('"') {
        if !check_printable(&format!("key: {value}")) {
            // Special characters in a double-quoted string are assumed to
            // have been backslash-escaped.
            return true;
        }
        if has_backslash_on_at_least_one_line_ending(elem, buffer) {
            return true;
        }
    }

    let mut scanner = Scanner::new(&format!("key: {value}"));
    // Remove the 5 first tokens corresponding to 'key: ' (StreamStart,
    // BlockMappingStart, Key, Scalar(key), Value).
    for _ in 0..5 {
        if scanner.get_token().is_err() {
            return true;
        }
    }
    let a = scanner.get_token();
    let b = scanner.get_token();
    match (a, b) {
        (Ok(Some(a)), Ok(Some(b))) => {
            if let TokenKind::Scalar {
                value: a_value,
                style: None,
                ..
            } = &a.kind
            {
                !(matches!(b.kind, TokenKind::BlockEnd) && a_value == value)
            } else {
                true
            }
        }
        _ => true,
    }
}

fn has_quoted_quotes(token_style: Option<char>, plain: bool, value: &str) -> bool {
    !plain
        && ((token_style == Some('\'') && value.contains('"'))
            || (token_style == Some('"') && value.contains('\'')))
}

pub fn check(
    conf: &Conf,
    elem: &TokenElem,
    buffer: &[char],
    ctx: &mut Context,
    problems: &mut Vec<LintProblem>,
) {
    let token = &elem.curr;
    let prev = elem.prev.as_ref();

    match token.kind {
        TokenKind::FlowMappingStart | TokenKind::FlowSequenceStart => {
            ctx.flow_nest_count += 1;
        }
        TokenKind::FlowMappingEnd | TokenKind::FlowSequenceEnd => {
            ctx.flow_nest_count -= 1;
        }
        _ => {}
    }

    let TokenKind::Scalar {
        value,
        plain,
        style,
    } = &token.kind
    else {
        return;
    };
    let style = *style;
    let plain = *plain;

    if !matches!(
        prev.map(|t| &t.kind),
        Some(TokenKind::BlockEntry)
            | Some(TokenKind::FlowEntry)
            | Some(TokenKind::FlowSequenceStart)
            | Some(TokenKind::Tag { .. })
            | Some(TokenKind::Value)
            | Some(TokenKind::Key)
    ) {
        return;
    }

    let node = if matches!(prev.map(|t| &t.kind), Some(TokenKind::Key)) {
        "key"
    } else {
        "value"
    };
    if node == "key" && !conf.check_keys {
        return;
    }

    // Ignore explicit types, e.g. !!str testtest or !!int 42
    if let Some(TokenKind::Tag { handle, .. }) = prev.map(|t| &t.kind)
        && handle.as_deref() == Some("!!")
    {
        return;
    }

    // Ignore numbers, booleans, etc.
    let tag = resolve_scalar_tag(value);
    if plain && tag != DEFAULT_SCALAR_TAG {
        return;
    }

    // Ignore multi-line strings.
    if !plain && matches!(style, Some('|') | Some('>')) {
        return;
    }

    let quote_type = conf.quote_type;

    let mut msg: Option<String> = None;
    match conf.required {
        Required::True => {
            // Quotes are mandatory and need to match config.
            if style.is_none()
                || !(quote_match(quote_type, style, ctx)
                    || (conf.allow_quoted_quotes && has_quoted_quotes(style, plain, value)))
            {
                msg = Some(format!(
                    "string {node} is not quoted with {} quotes",
                    quote_type.as_str()
                ));
            }
        }
        Required::False => {
            // Quotes are not mandatory but when used need to match config.
            if style.is_some()
                && !quote_match(quote_type, style, ctx)
                && !(conf.allow_quoted_quotes && has_quoted_quotes(style, plain, value))
            {
                msg = Some(format!(
                    "string {node} is not quoted with {} quotes",
                    quote_type.as_str()
                ));
            } else if style.is_none() {
                let is_extra_required = conf.extra_required.iter().any(|r| r.is_match(value));
                if is_extra_required {
                    msg = Some(format!("string {node} is not quoted"));
                }
            }
        }
        Required::OnlyWhenNeeded => {
            // Quotes are not strictly needed here.
            if style.is_some()
                && tag == DEFAULT_SCALAR_TAG
                && !value.is_empty()
                && !quotes_are_needed(elem, buffer, ctx.flow_nest_count > 0)
            {
                let is_extra_required = conf.extra_required.iter().any(|r| r.is_match(value));
                let is_extra_allowed = conf.extra_allowed.iter().any(|r| r.is_match(value));
                if !(is_extra_required || is_extra_allowed) {
                    msg = Some(format!(
                        "string {node} is redundantly quoted with {} quotes",
                        quote_type.as_str()
                    ));
                }
            }
            // But when used need to match config.
            else if style.is_some()
                && !quote_match(quote_type, style, ctx)
                && !(conf.allow_quoted_quotes && has_quoted_quotes(style, plain, value))
            {
                msg = Some(format!(
                    "string {node} is not quoted with {} quotes",
                    quote_type.as_str()
                ));
            } else if style.is_none() {
                let is_extra_required = !conf.extra_required.is_empty()
                    && conf.extra_required.iter().any(|r| r.is_match(value));
                if is_extra_required {
                    msg = Some(format!("string {node} is not quoted"));
                }
            }
        }
    }

    if let Some(msg) = msg {
        problems.push(problem_at(
            token.start_mark.line + 1,
            token.start_mark.column + 1,
            msg,
        ));
    }
}
