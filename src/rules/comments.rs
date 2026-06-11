//! `comments`: control the position and formatting of comments.

use crate::linter::{Comment, LintProblem};
use crate::rules::common::problem_at;

pub const ID: &str = "comments";

#[derive(Debug, Clone)]
pub struct Conf {
    pub require_starting_space: bool,
    pub ignore_shebangs: bool,
    pub min_spaces_from_content: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            require_starting_space: true,
            ignore_shebangs: true,
            min_spaces_from_content: 2,
        }
    }
}

pub fn check(conf: &Conf, comment: &Comment, buffer: &[char], problems: &mut Vec<LintProblem>) {
    if conf.min_spaces_from_content != -1
        && comment.is_inline(buffer)
        && let Some(token_before) = &comment.token_before
        && (comment.pointer as i64 - token_before.end_mark.pointer as i64)
            < conf.min_spaces_from_content
    {
        problems.push(problem_at(
            comment.line_no,
            comment.column_no,
            format!(
                "too few spaces before comment: expected {}",
                conf.min_spaces_from_content
            ),
        ));
    }

    if conf.require_starting_space {
        let mut text_start = comment.pointer + 1;
        while buffer.get(text_start) == Some(&'#') {
            text_start += 1;
        }
        if text_start < buffer.len() {
            let ch = buffer[text_start];
            if conf.ignore_shebangs && comment.line_no == 1 && comment.column_no == 1 && ch == '!' {
                return;
            }
            // We can test for both \r and \r\n just by checking first char.
            if !matches!(ch, ' ' | '\n' | '\r' | '\0') {
                let column = comment.column_no + text_start - comment.pointer;
                problems.push(problem_at(
                    comment.line_no,
                    column,
                    "missing starting space in comment".to_string(),
                ));
            }
        }
    }
}
