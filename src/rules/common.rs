//! Helpers shared by token rules.

use crate::linter::{Level, LintProblem};
use crate::pyyaml::tokens::{Token, TokenKind};

pub fn problem_at(line: usize, column: usize, desc: String) -> LintProblem {
    LintProblem {
        line,
        column,
        desc,
        rule: None,
        level: Level::Error,
    }
}

pub fn spaces_after(
    token: &Token,
    next: Option<&Token>,
    min: i64,
    max: i64,
    min_desc: &str,
    max_desc: &str,
) -> Option<LintProblem> {
    if let Some(next) = next
        && token.end_mark.line == next.start_mark.line
    {
        let spaces = next.start_mark.pointer as i64 - token.end_mark.pointer as i64;
        if max != -1 && spaces > max {
            return Some(problem_at(
                token.start_mark.line + 1,
                next.start_mark.column,
                max_desc.to_string(),
            ));
        } else if min != -1 && spaces < min {
            return Some(problem_at(
                token.start_mark.line + 1,
                next.start_mark.column + 1,
                min_desc.to_string(),
            ));
        }
    }
    None
}

pub fn spaces_before(
    token: &Token,
    prev: Option<&Token>,
    buffer: &[char],
    min: i64,
    max: i64,
    min_desc: &str,
    max_desc: &str,
) -> Option<LintProblem> {
    if let Some(prev) = prev
        && prev.end_mark.line == token.start_mark.line
            // Discard tokens (only scalars?) that end at the start of next line
            && (prev.end_mark.pointer == 0
                || buffer[prev.end_mark.pointer - 1] != '\n')
    {
        let spaces = token.start_mark.pointer as i64 - prev.end_mark.pointer as i64;
        if max != -1 && spaces > max {
            return Some(problem_at(
                token.start_mark.line + 1,
                token.start_mark.column,
                max_desc.to_string(),
            ));
        } else if min != -1 && spaces < min {
            return Some(problem_at(
                token.start_mark.line + 1,
                token.start_mark.column + 1,
                min_desc.to_string(),
            ));
        }
    }
    None
}

pub fn get_line_indent(token: &Token, buffer: &[char]) -> usize {
    let start = buffer[..token.start_mark.pointer]
        .iter()
        .rposition(|&c| c == '\n')
        .map(|p| p + 1)
        .unwrap_or(0);
    let mut content = start;
    while content < buffer.len() && buffer[content] == ' ' {
        content += 1;
    }
    content - start
}

/// With PyYAML, scalar tokens often end on a next line.
pub fn get_real_end_line(token: &Token, buffer: &[char]) -> usize {
    let mut end_line = token.end_mark.line + 1;

    if !matches!(token.kind, TokenKind::Scalar { .. }) {
        return end_line;
    }

    let mut pos = token.end_mark.pointer as i64 - 1;
    while pos >= token.start_mark.pointer as i64 - 1
        && pos >= 0
        && matches!(
            buffer[pos as usize],
            ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C'
        )
    {
        if buffer[pos as usize] == '\n' {
            end_line -= 1;
        }
        pos -= 1;
    }
    end_line
}

pub fn is_explicit_key(token: &Token, buffer: &[char]) -> bool {
    // explicit key:
    //   ? key
    //   : v
    token.start_mark.pointer < token.end_mark.pointer && buffer[token.start_mark.pointer] == '?'
}
