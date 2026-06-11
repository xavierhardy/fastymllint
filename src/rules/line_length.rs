//! `line-length`: set a limit to lines length.

use crate::linter::{Line, LintProblem};
use crate::pyyaml::scanner::Scanner;
use crate::pyyaml::tokens::TokenKind;
use crate::rules::common::problem_at;

pub const ID: &str = "line-length";

#[derive(Debug, Clone)]
pub struct Conf {
    pub max: i64,
    pub allow_non_breakable_words: bool,
    pub allow_non_breakable_inline_mappings: bool,
}

impl Default for Conf {
    fn default() -> Self {
        Self {
            max: 80,
            allow_non_breakable_words: true,
            allow_non_breakable_inline_mappings: false,
        }
    }
}

fn check_inline_mapping(line_content: &str) -> bool {
    let chars: Vec<char> = line_content.chars().collect();
    let mut scanner = Scanner::new(line_content);
    loop {
        match scanner.get_token() {
            Ok(Some(token)) => {
                if matches!(token.kind, TokenKind::BlockMappingStart) {
                    loop {
                        match scanner.get_token() {
                            Ok(Some(inner)) => {
                                if matches!(inner.kind, TokenKind::Value) {
                                    match scanner.get_token() {
                                        Ok(Some(t)) => {
                                            if let TokenKind::Scalar { .. } = t.kind {
                                                let from = t.start_mark.column.min(chars.len());
                                                return !chars[from..].contains(&' ');
                                            }
                                            // Not a scalar: keep scanning.
                                        }
                                        Ok(None) => return false,
                                        Err(_) => return false,
                                    }
                                }
                            }
                            Ok(None) => return false,
                            Err(_) => return false,
                        }
                    }
                }
            }
            Ok(None) => return false,
            Err(_) => return false,
        }
    }
}

pub fn check(conf: &Conf, line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
    let length = (line.end - line.start) as i64;
    if length > conf.max {
        let allow_non_breakable_words =
            conf.allow_non_breakable_words || conf.allow_non_breakable_inline_mappings;
        if allow_non_breakable_words {
            let mut start = line.start;
            while start < line.end && buffer[start] == ' ' {
                start += 1;
            }

            if start != line.end {
                if buffer[start] == '#' {
                    while buffer.get(start) == Some(&'#') {
                        start += 1;
                    }
                    start += 1;
                } else if buffer[start] == '-' {
                    start += 2;
                }

                // No space found between start and the end of the line (also
                // the case when start was advanced past the end).
                let from = start.min(line.end);
                let has_space = buffer[from..line.end].contains(&' ');
                if !has_space {
                    return;
                }

                if conf.allow_non_breakable_inline_mappings {
                    let content: String = buffer[line.start..line.end].iter().collect();
                    if check_inline_mapping(&content) {
                        return;
                    }
                }
            }
        }

        problems.push(problem_at(
            line.line_no,
            (conf.max + 1) as usize,
            format!("line too long ({} > {} characters)", length, conf.max),
        ));
    }
}
