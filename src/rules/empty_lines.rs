//! `empty-lines`: set a maximal number of allowed consecutive blank lines.

use crate::linter::{Line, LintProblem};
use crate::rules::common::problem_at;

pub const ID: &str = "empty-lines";

#[derive(Debug, Clone)]
pub struct Conf {
    pub max: i64,
    pub max_start: i64,
    pub max_end: i64,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max: 2,
            max_start: 0,
            max_end: 0,
        }
    }
}

fn buffer_eq(buffer: &[char], start: usize, s: &str) -> bool {
    s.chars()
        .enumerate()
        .all(|(i, ch)| buffer.get(start + i).copied() == Some(ch))
}

// Parallel branches are kept separate on purpose: each one corresponds to
// a distinct check documented for this rule.
#[allow(clippy::if_same_then_else)]
pub fn check(conf: &Conf, line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let raw_len = buffer.len().saturating_sub(1);
    if line.start == line.end && line.end < raw_len {
        // Only alert on the last blank line of a series.
        if line.end + 2 <= raw_len && buffer_eq(buffer, line.end, "\n\n") {
            return;
        } else if line.end + 4 <= raw_len && buffer_eq(buffer, line.end, "\r\n\r\n") {
            return;
        }

        let mut blank_lines: i64 = 0;

        let mut start = line.start;
        while start >= 2 && buffer[start - 2] == '\r' && buffer[start - 1] == '\n' {
            blank_lines += 1;
            start -= 2;
        }
        while start >= 1 && buffer[start - 1] == '\n' {
            blank_lines += 1;
            start -= 1;
        }

        let mut max = conf.max;

        // Special case: start of document.
        if start == 0 {
            blank_lines += 1; // first line doesn't have a preceding \n
            max = conf.max_start;
        }

        // Special case: end of document.
        if (line.end == raw_len.wrapping_sub(1) && buffer[line.end] == '\n')
            || (raw_len >= 2 && line.end == raw_len - 2 && buffer_eq(buffer, line.end, "\r\n"))
        {
            // Allow the exception of the one-byte file containing '\n'.
            if line.end == 0 {
                return;
            }

            max = conf.max_end;
        }

        if blank_lines > max {
            problems.push(problem_at(
                line.line_no,
                1,
                format!("too many blank lines ({blank_lines} > {max})"),
            ));
        }
    }
}
