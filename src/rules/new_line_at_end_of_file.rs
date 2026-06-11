//! `new-line-at-end-of-file`: require a new line character at end of files.

use crate::linter::{Line, LintProblem};
use crate::rules::common::problem_at;

pub const ID: &str = "new-line-at-end-of-file";

pub fn check(line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let raw_len = buffer.len().saturating_sub(1);
    if line.end == raw_len && line.end > line.start {
        problems.push(problem_at(
            line.line_no,
            line.end - line.start + 1,
            "no new line character at the end of file".to_string(),
        ));
    }
}
