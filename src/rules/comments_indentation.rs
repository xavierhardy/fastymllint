//! `comments-indentation`: force comments to be indented like content.

use crate::linter::{Comment, LintProblem};
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::{get_line_indent, problem_at};

pub const ID: &str = "comments-indentation";

// Case A:
//
//     prev: line:
//       # commented line
//       current: line
//
// Case B:
//
//       prev: line
//       # commented line 1
//     # commented line 2
//     current: line

pub fn check(comment: &Comment, buffer: &[char], problems: &mut Vec<LintProblem>) {
    // Only check block comments.
    if let Some(token_before) = &comment.token_before
        && !matches!(token_before.kind, TokenKind::StreamStart)
        && token_before.end_mark.line + 1 == comment.line_no
    {
        return;
    }

    let Some(token_after) = &comment.token_after else {
        return;
    };

    let mut next_line_indent = token_after.start_mark.column as i64;
    if matches!(token_after.kind, TokenKind::StreamEnd) {
        next_line_indent = 0;
    }

    let mut prev_line_indent: i64 = match &comment.token_before {
        Some(token) if !matches!(token.kind, TokenKind::StreamStart) => {
            get_line_indent(token, buffer) as i64
        }
        _ => 0,
    };

    // In the following case only the next line indent is valid:
    //     list:
    //         # comment
    //         - 1
    //         - 2
    prev_line_indent = prev_line_indent.max(next_line_indent);

    // If two indents are valid but a previous comment went back to normal
    // indent, force the next ones to do the same.
    if let Some((col, inline)) = comment.comment_before
        && !inline
    {
        prev_line_indent = col as i64 - 1;
    }

    let col = comment.column_no as i64 - 1;
    if col != prev_line_indent && col != next_line_indent {
        problems.push(problem_at(
            comment.line_no,
            comment.column_no,
            "comment not indented like content".to_string(),
        ));
    }
}
