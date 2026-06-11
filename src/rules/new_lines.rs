//! `new-lines`: force the type of new line characters.

use crate::linter::{Line, LintProblem};
use crate::rules::common::problem_at;

pub const ID: &str = "new-lines";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NewLineType {
    Unix,
    Dos,
    Platform,
}

#[derive(Debug, Clone)]
pub struct Conf {
    pub r#type: NewLineType,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            r#type: NewLineType::Unix,
        }
    }
}

pub fn check(conf: &Conf, line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let newline_char: &str = match conf.r#type {
        NewLineType::Unix => "\n",
        NewLineType::Dos => "\r\n",
        NewLineType::Platform => {
            if cfg!(windows) {
                "\r\n"
            } else {
                "\n"
            }
        }
    };

    // Only check the first line; the buffer includes the '\0' sentinel.
    let raw_len = buffer.len().saturating_sub(1);
    if line.start == 0 && raw_len > line.end {
        let mut matches = true;
        for (i, ch) in newline_char.chars().enumerate() {
            if buffer.get(line.end + i).copied() != Some(ch) {
                matches = false;
                break;
            }
        }
        if !matches {
            let c = match newline_char {
                "\n" => "\\n",
                _ => "\\r\\n",
            };
            problems.push(problem_at(
                1,
                line.end - line.start + 1,
                format!("wrong new line character: expected {c}"),
            ));
        }
    }
}
