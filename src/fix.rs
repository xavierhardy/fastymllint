//! Auto-fix and auto-format engine.
//!
//! Fixes are split in two classes:
//! - safe: whitespace/marker-only edits that cannot change the meaning of
//!   the YAML document (trailing spaces, newlines, blank lines, spacing
//!   around punctuation, comment spacing/indent, document markers);
//! - unsafe (`--unsafe`): edits that may change how a document is parsed by
//!   some tools (truthy normalization, quoting octal values, adding/removing
//!   string quotes, re-indenting lines).
//!
//! The engine repeatedly lints and applies textual edits derived from the
//! reported problems until a fixed point is reached.

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

use crate::config::YamlLintConfig;
use crate::linter::{self, LintProblem};
use crate::rules::RuleOptions;

const MAX_PASSES: usize = 25;

/// Whether a rule's fixer is safe (style-only) or unsafe.
fn fix_is_safe(rule: &str) -> bool {
    !matches!(
        rule,
        "truthy" | "octal-values" | "quoted-strings" | "indentation"
    )
}

struct Doc {
    lines: Vec<String>,
    ending: String,
    final_newline: bool,
}

impl Doc {
    fn parse(content: &str) -> Self {
        let ending = if content.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let final_newline = content.ends_with('\n');
        let mut lines: Vec<String> = content
            .split('\n')
            .map(|l| l.strip_suffix('\r').unwrap_or(l).to_string())
            .collect();
        if final_newline {
            lines.pop();
        }
        Self {
            lines,
            ending: ending.to_string(),
            final_newline,
        }
    }

    fn render(&self) -> String {
        let mut out = self.lines.join(&self.ending);
        if self.final_newline {
            out.push_str(&self.ending);
        }
        out
    }
}

#[derive(Debug)]
enum FixOp {
    ReplaceLine { line: usize, text: String },
    DeleteLines { start: usize, count: usize },
    InsertLine { at: usize, text: String },
    SetEnding { dos: bool },
    EnsureFinalNewline,
}

fn line_chars(doc: &Doc, line: usize) -> Option<Vec<char>> {
    doc.lines.get(line - 1).map(|l| l.chars().collect())
}

/// Shrink the run of spaces containing the character at `idx` down to
/// `target` spaces. Returns the new line if a change was made.
fn shrink_space_run(chars: &[char], idx: usize, target: i64) -> Option<String> {
    if target < 0 {
        return None;
    }
    let target = target as usize;
    if idx >= chars.len() || chars[idx] != ' ' {
        return None;
    }
    let mut start = idx;
    while start > 0 && chars[start - 1] == ' ' {
        start -= 1;
    }
    let mut end = idx;
    while end + 1 < chars.len() && chars[end + 1] == ' ' {
        end += 1;
    }
    let run = end - start + 1;
    if run <= target {
        return None;
    }
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    out.extend_from_slice(&chars[..start]);
    out.extend(std::iter::repeat_n(' ', target));
    out.extend_from_slice(&chars[end + 1..]);
    Some(out.into_iter().collect())
}

/// Insert spaces before the character at `idx` so that `min` spaces precede
/// it.
fn grow_space_run(chars: &[char], idx: usize, min: i64) -> Option<String> {
    if min < 0 || idx > chars.len() {
        return None;
    }
    let min = min as usize;
    let mut existing = 0;
    let mut i = idx;
    while i > 0 && chars[i - 1] == ' ' {
        existing += 1;
        i -= 1;
    }
    if existing >= min {
        return None;
    }
    let mut out: Vec<char> = Vec::with_capacity(chars.len() + min - existing);
    out.extend_from_slice(&chars[..idx]);
    out.extend(std::iter::repeat_n(' ', min - existing));
    out.extend_from_slice(&chars[idx..]);
    Some(out.into_iter().collect())
}

fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|&c| c == ' ').count()
}

static WRONG_INDENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^wrong indentation: expected (\d+) but found (\d+)$").unwrap());
static QUOTED_VALUE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""(.*)"$"#).unwrap());

struct RuleConfs<'a> {
    confs: Vec<(&'static str, &'a RuleOptions)>,
}

impl<'a> RuleConfs<'a> {
    fn get(&self, id: &str) -> Option<&'a RuleOptions> {
        self.confs
            .iter()
            .find(|(rid, _)| *rid == id)
            .map(|(_, c)| *c)
    }
}

fn fixer_for(
    problem: &LintProblem,
    doc: &Doc,
    confs: &RuleConfs,
    allow_unsafe: bool,
) -> Option<FixOp> {
    let rule = problem.rule?;
    if !allow_unsafe && !fix_is_safe(rule) {
        return None;
    }

    match rule {
        "trailing-spaces" => {
            let line = doc.lines.get(problem.line - 1)?;
            let trimmed = line.trim_end();
            if trimmed.len() != line.len() {
                Some(FixOp::ReplaceLine {
                    line: problem.line,
                    text: trimmed.to_string(),
                })
            } else {
                None
            }
        }
        "new-line-at-end-of-file" => Some(FixOp::EnsureFinalNewline),
        "new-lines" => {
            let dos = match confs.get(rule) {
                Some(RuleOptions::NewLines(conf)) => match conf.r#type {
                    crate::rules::new_lines::NewLineType::Dos => true,
                    crate::rules::new_lines::NewLineType::Unix => false,
                    crate::rules::new_lines::NewLineType::Platform => cfg!(windows),
                },
                _ => false,
            };
            Some(FixOp::SetEnding { dos })
        }
        "empty-lines" => {
            let conf = match confs.get(rule) {
                Some(RuleOptions::EmptyLines(c)) => c,
                _ => return None,
            };
            // problem.line is the last blank line of a series.
            let last = problem.line;
            if last > doc.lines.len() || !doc.lines[last - 1].is_empty() {
                return None;
            }
            let mut first = last;
            while first > 1 && doc.lines[first - 2].is_empty() {
                first -= 1;
            }
            let run = (last - first + 1) as i64;
            let allowed = if first == 1 {
                conf.max_start
            } else if last == doc.lines.len() {
                conf.max_end
            } else {
                conf.max
            };
            let excess = run - allowed.max(0);
            if excess <= 0 {
                return None;
            }
            Some(FixOp::DeleteLines {
                start: first,
                count: excess as usize,
            })
        }
        "document-start" => {
            let present = match confs.get(rule) {
                Some(RuleOptions::DocumentStart(c)) => c.present,
                _ => true,
            };
            if present {
                Some(FixOp::InsertLine {
                    at: problem.line,
                    text: "---".to_string(),
                })
            } else {
                let line = doc.lines.get(problem.line - 1)?;
                let rest = line.strip_prefix("---")?;
                let rest = rest.strip_prefix(' ').unwrap_or(rest);
                if rest.is_empty() {
                    Some(FixOp::DeleteLines {
                        start: problem.line,
                        count: 1,
                    })
                } else {
                    Some(FixOp::ReplaceLine {
                        line: problem.line,
                        text: rest.to_string(),
                    })
                }
            }
        }
        "document-end" => {
            let present = match confs.get(rule) {
                Some(RuleOptions::DocumentEnd(c)) => c.present,
                _ => true,
            };
            if present {
                let at = if problem.line <= doc.lines.len()
                    && doc.lines[problem.line - 1].trim_start().starts_with("---")
                {
                    problem.line
                } else {
                    doc.lines.len() + 1
                };
                Some(FixOp::InsertLine {
                    at,
                    text: "...".to_string(),
                })
            } else {
                let line = doc.lines.get(problem.line - 1)?;
                if line.trim() == "..." {
                    Some(FixOp::DeleteLines {
                        start: problem.line,
                        count: 1,
                    })
                } else {
                    None
                }
            }
        }
        "comments" => {
            let chars = line_chars(doc, problem.line)?;
            if problem.desc == "missing starting space in comment" {
                let idx = problem.column - 1;
                if idx > chars.len() {
                    return None;
                }
                let mut out: Vec<char> = Vec::with_capacity(chars.len() + 1);
                out.extend_from_slice(&chars[..idx]);
                out.push(' ');
                out.extend_from_slice(&chars[idx..]);
                Some(FixOp::ReplaceLine {
                    line: problem.line,
                    text: out.into_iter().collect(),
                })
            } else if problem.desc.starts_with("too few spaces before comment") {
                let min = match confs.get(rule) {
                    Some(RuleOptions::Comments(c)) => c.min_spaces_from_content,
                    _ => 2,
                };
                let idx = problem.column - 1; // position of '#'
                grow_space_run(&chars, idx, min).map(|text| FixOp::ReplaceLine {
                    line: problem.line,
                    text,
                })
            } else {
                None
            }
        }
        "comments-indentation" => {
            let line = doc.lines.get(problem.line - 1)?;
            if !line.trim_start().starts_with('#') {
                return None;
            }
            // Align to the next non-blank, non-comment line's indent.
            let mut target = 0;
            for next in doc.lines.iter().skip(problem.line) {
                let trimmed = next.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                target = leading_spaces(next);
                break;
            }
            let current = leading_spaces(line);
            if current == target {
                return None;
            }
            let text = format!("{}{}", " ".repeat(target), line.trim_start());
            Some(FixOp::ReplaceLine {
                line: problem.line,
                text,
            })
        }
        "colons" | "hyphens" | "commas" | "braces" | "brackets" => {
            let chars = line_chars(doc, problem.line)?;
            let desc = problem.desc.as_str();
            let (is_min, target): (bool, i64) = match (rule, confs.get(rule)) {
                ("colons", Some(RuleOptions::Colons(c))) => {
                    if desc == "too many spaces before colon" {
                        (false, c.max_spaces_before)
                    } else {
                        (false, c.max_spaces_after)
                    }
                }
                ("hyphens", Some(RuleOptions::Hyphens(c))) => (false, c.max_spaces_after),
                ("commas", Some(RuleOptions::Commas(c))) => {
                    if desc == "too many spaces before comma" {
                        (false, c.max_spaces_before)
                    } else if desc == "too few spaces after comma" {
                        (true, c.min_spaces_after)
                    } else {
                        (false, c.max_spaces_after)
                    }
                }
                ("braces", Some(RuleOptions::Braces(c))) => {
                    if desc.starts_with("too few") {
                        if desc.contains("empty") && c.min_spaces_inside_empty != -1 {
                            (true, c.min_spaces_inside_empty)
                        } else {
                            (true, c.min_spaces_inside)
                        }
                    } else if desc.contains("empty") && c.max_spaces_inside_empty != -1 {
                        (false, c.max_spaces_inside_empty)
                    } else {
                        (false, c.max_spaces_inside)
                    }
                }
                ("brackets", Some(RuleOptions::Brackets(c))) => {
                    if desc.starts_with("too few") {
                        if desc.contains("empty") && c.min_spaces_inside_empty != -1 {
                            (true, c.min_spaces_inside_empty)
                        } else {
                            (true, c.min_spaces_inside)
                        }
                    } else if desc.contains("empty") && c.max_spaces_inside_empty != -1 {
                        (false, c.max_spaces_inside_empty)
                    } else {
                        (false, c.max_spaces_inside)
                    }
                }
                _ => return None,
            };

            if is_min {
                // min problems point at the character that needs spaces
                // before it (1-based column of that char).
                let idx = problem.column - 1;
                grow_space_run(&chars, idx, target).map(|text| FixOp::ReplaceLine {
                    line: problem.line,
                    text,
                })
            } else {
                // max problems point just past the space run.
                let idx = problem.column.checked_sub(1)?;
                shrink_space_run(&chars, idx, target).map(|text| FixOp::ReplaceLine {
                    line: problem.line,
                    text,
                })
            }
        }
        "truthy" => {
            let conf = match confs.get(rule) {
                Some(RuleOptions::Truthy(c)) => c,
                _ => return None,
            };
            let chars = line_chars(doc, problem.line)?;
            let start = problem.column - 1;
            if start >= chars.len() {
                return None;
            }
            let mut end = start;
            while end < chars.len() && chars[end].is_ascii_alphabetic() {
                end += 1;
            }
            let word: String = chars[start..end].iter().collect();
            let truthy = matches!(word.to_lowercase().as_str(), "yes" | "on" | "true");
            let falsy = matches!(word.to_lowercase().as_str(), "no" | "off" | "false");
            if !truthy && !falsy {
                return None;
            }
            let wanted = if truthy { "true" } else { "false" };
            let replacement = if conf.allowed_values.iter().any(|v| v == wanted) {
                wanted.to_string()
            } else {
                // Find an allowed value with the same truthiness.
                conf.allowed_values
                    .iter()
                    .find(|v| {
                        let lower = v.to_lowercase();
                        if truthy {
                            matches!(lower.as_str(), "yes" | "on" | "true")
                        } else {
                            matches!(lower.as_str(), "no" | "off" | "false")
                        }
                    })?
                    .clone()
            };
            let mut out: Vec<char> = Vec::new();
            out.extend_from_slice(&chars[..start]);
            out.extend(replacement.chars());
            out.extend_from_slice(&chars[end..]);
            Some(FixOp::ReplaceLine {
                line: problem.line,
                text: out.into_iter().collect(),
            })
        }
        "octal-values" => {
            let value = QUOTED_VALUE.captures(&problem.desc)?.get(1)?.as_str();
            let chars = line_chars(doc, problem.line)?;
            let end = problem.column - 1; // end of token (0-based)
            let len = value.chars().count();
            let start = end.checked_sub(len)?;
            let found: String = chars.get(start..end)?.iter().collect();
            if found != value {
                return None;
            }
            let mut out: Vec<char> = Vec::new();
            out.extend_from_slice(&chars[..start]);
            out.push('\'');
            out.extend(value.chars());
            out.push('\'');
            out.extend_from_slice(&chars[end..]);
            Some(FixOp::ReplaceLine {
                line: problem.line,
                text: out.into_iter().collect(),
            })
        }
        "quoted-strings" => {
            let chars = line_chars(doc, problem.line)?;
            let start = problem.column - 1;
            if start >= chars.len() {
                return None;
            }
            let desc = problem.desc.as_str();
            if desc.contains("redundantly quoted") {
                let quote = chars[start];
                if quote != '\'' && quote != '"' {
                    return None;
                }
                let mut end = start + 1;
                while end < chars.len() && chars[end] != quote {
                    if quote == '"' && chars[end] == '\\' {
                        return None; // escapes: leave alone
                    }
                    if quote == '\'' && chars[end] == '\'' {
                        return None;
                    }
                    end += 1;
                }
                if end >= chars.len() {
                    return None; // multi-line string
                }
                let inner: String = chars[start + 1..end].iter().collect();
                let mut out: Vec<char> = Vec::new();
                out.extend_from_slice(&chars[..start]);
                out.extend(inner.chars());
                out.extend_from_slice(&chars[end + 1..]);
                Some(FixOp::ReplaceLine {
                    line: problem.line,
                    text: out.into_iter().collect(),
                })
            } else if desc.contains("is not quoted") {
                let conf = match confs.get(rule) {
                    Some(RuleOptions::QuotedStrings(c)) => c,
                    _ => return None,
                };
                if chars[start] == '\'' || chars[start] == '"' {
                    // Wrong quote type: skip (rewriting quotes is riskier).
                    return None;
                }
                // Take the scalar up to an inline comment or end of line.
                let mut end = chars.len();
                for i in start..chars.len() {
                    if chars[i] == '#' && i > start && chars[i - 1] == ' ' {
                        end = i;
                        break;
                    }
                }
                while end > start && chars[end - 1] == ' ' {
                    end -= 1;
                }
                if end <= start {
                    return None;
                }
                let value: String = chars[start..end].iter().collect();
                // Skip values that themselves contain quotes or flow
                // indicators: too risky for a textual fix.
                if value.contains('\'')
                    || value.contains('"')
                    || value.contains('\\')
                    || value.contains(',')
                    || value.contains('{')
                    || value.contains('}')
                    || value.contains('[')
                    || value.contains(']')
                    || value.contains(": ")
                    || value.ends_with(':')
                {
                    return None;
                }
                let quote = match conf.quote_type {
                    crate::rules::quoted_strings::QuoteType::Single => '\'',
                    _ => '"',
                };
                let mut out: Vec<char> = Vec::new();
                out.extend_from_slice(&chars[..start]);
                out.push(quote);
                out.extend(value.chars());
                out.push(quote);
                out.extend_from_slice(&chars[end..]);
                Some(FixOp::ReplaceLine {
                    line: problem.line,
                    text: out.into_iter().collect(),
                })
            } else {
                None
            }
        }
        "indentation" => {
            let caps = WRONG_INDENT.captures(&problem.desc)?;
            let expected: usize = caps.get(1)?.as_str().parse().ok()?;
            let line = doc.lines.get(problem.line - 1)?;
            let found = leading_spaces(line);
            if found == expected {
                return None;
            }
            let text = format!("{}{}", " ".repeat(expected), line.trim_start());
            Some(FixOp::ReplaceLine {
                line: problem.line,
                text,
            })
        }
        _ => None,
    }
}

/// Apply one pass worth of fix operations. Returns the number applied.
fn apply_ops(doc: &mut Doc, ops: Vec<FixOp>) -> usize {
    let mut applied = 0;
    let mut touched_lines: HashSet<usize> = HashSet::new();
    let mut structural: Vec<FixOp> = Vec::new();

    for op in ops {
        match op {
            FixOp::SetEnding { dos } => {
                let wanted = if dos { "\r\n" } else { "\n" };
                if doc.ending != wanted {
                    doc.ending = wanted.to_string();
                    applied += 1;
                }
            }
            FixOp::EnsureFinalNewline => {
                if !doc.final_newline {
                    doc.final_newline = true;
                    applied += 1;
                }
            }
            FixOp::ReplaceLine { line, text } => {
                if touched_lines.insert(line)
                    && line >= 1
                    && line <= doc.lines.len()
                    && doc.lines[line - 1] != text
                {
                    doc.lines[line - 1] = text;
                    applied += 1;
                }
            }
            op @ (FixOp::DeleteLines { .. } | FixOp::InsertLine { .. }) => {
                structural.push(op);
            }
        }
    }

    // Apply structural ops bottom-up so earlier line numbers stay valid.
    structural.sort_by_key(|op| {
        std::cmp::Reverse(match op {
            FixOp::DeleteLines { start, .. } => *start,
            FixOp::InsertLine { at, .. } => *at,
            _ => 0,
        })
    });
    for op in structural {
        match op {
            FixOp::DeleteLines { start, count } => {
                if start >= 1 && start - 1 + count <= doc.lines.len() {
                    doc.lines.drain(start - 1..start - 1 + count);
                    applied += 1;
                }
            }
            FixOp::InsertLine { at, text } => {
                let idx = (at - 1).min(doc.lines.len());
                doc.lines.insert(idx, text);
                applied += 1;
            }
            _ => {}
        }
    }

    applied
}

pub struct FixResult {
    pub fixed: String,
    pub changed: bool,
    /// Problems remaining after fixing.
    pub remaining: Vec<LintProblem>,
}

/// Fix as many problems as possible. `allow_unsafe` enables fixes that may
/// change document semantics.
pub fn fix_content(content: &str, conf: &YamlLintConfig, allow_unsafe: bool) -> FixResult {
    let enabled = conf.enabled_rules(None);
    let confs = RuleConfs {
        confs: enabled.iter().map(|r| (r.id, r.conf)).collect(),
    };

    let mut current = content.to_string();
    for _pass in 0..MAX_PASSES {
        let problems = linter::run(&current, conf, None);
        let mut doc = Doc::parse(&current);
        let ops: Vec<FixOp> = problems
            .iter()
            .filter_map(|p| fixer_for(p, &doc, &confs, allow_unsafe))
            .collect();
        if ops.is_empty() {
            break;
        }
        if apply_ops(&mut doc, ops) == 0 {
            break;
        }
        let rendered = doc.render();
        if rendered == current {
            break;
        }
        current = rendered;
    }

    let remaining = linter::run(&current, conf, None);
    FixResult {
        changed: current != content,
        fixed: current,
        remaining,
    }
}

/// Produce a unified diff between original and fixed content.
pub fn unified_diff(path: &str, original: &str, fixed: &str) -> String {
    let diff = similar::TextDiff::from_lines(original, fixed);
    diff.unified_diff()
        .context_radius(3)
        .header(&format!("a/{path}"), &format!("b/{path}"))
        .to_string()
}
