//! `trailing-spaces`: forbid trailing spaces at the end of lines.

use crate::linter::{Line, LintProblem};
use crate::rules::common::problem_at;

pub const ID: &str = "trailing-spaces";

pub fn check(line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
    if line.end == 0 {
        return;
    }

    // YAML recognizes two white space characters: space and tab.
    let mut pos = line.end;
    while pos > line.start && matches!(buffer[pos - 1], ' ' | '\t' | '\n' | '\r' | '\x0B' | '\x0C')
    {
        pos -= 1;
    }

    if pos != line.end && matches!(buffer[pos], ' ' | '\t') {
        problems.push(problem_at(
            line.line_no,
            pos - line.start + 1,
            "trailing spaces".to_string(),
        ));
    }
}
