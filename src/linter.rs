//! The linting pipeline: token / comment / line generation, problem
//! collection, inline `# yamllint disable...` directives and syntax-error
//! merging.

use crate::config::YamlLintConfig;
use crate::pyyaml::parser::Parser;
use crate::pyyaml::scanner::Scanner;
use crate::pyyaml::tokens::{Token, TokenKind};
use crate::rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warning,
    Error,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Warning => "warning",
            Level::Error => "error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LintProblem {
    /// Line on which the problem was found (starting at 1).
    pub line: usize,
    /// Column on which the problem was found (starting at 1).
    pub column: usize,
    /// Human-readable description of the problem.
    pub desc: String,
    /// Identifier of the rule that detected the problem.
    pub rule: Option<&'static str>,
    pub level: Level,
}

impl LintProblem {
    pub fn message(&self) -> String {
        match self.rule {
            Some(rule) => format!("{} ({})", self.desc, rule),
            None => self.desc.clone(),
        }
    }
}

/// A physical line; `start`/`end` are character indices into the buffer
/// (excluding the trailing `'\0'` sentinel and the line break).
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub line_no: usize,
    pub start: usize,
    pub end: usize,
}

impl Line {
    pub fn content<'a>(&self, buffer: &'a [char]) -> &'a [char] {
        &buffer[self.start..self.end]
    }
}

/// A comment found between tokens.
#[derive(Debug, Clone)]
pub struct Comment {
    pub line_no: usize,
    /// 1-based column.
    pub column_no: usize,
    /// Character index of the `#` in the buffer.
    pub pointer: usize,
    pub token_before: Option<Token>,
    pub token_after: Option<Token>,
    /// (column_no, is_inline) of the previous comment in the same gap.
    pub comment_before: Option<(usize, bool)>,
}

impl Comment {
    /// The comment text from `#` to the end of the line.
    pub fn text(&self, buffer: &[char]) -> String {
        let mut end = self.pointer;
        while end < buffer.len() && buffer[end] != '\n' && buffer[end] != '\0' {
            end += 1;
        }
        buffer[self.pointer..end].iter().collect()
    }

    pub fn is_inline(&self, buffer: &[char]) -> bool {
        match &self.token_before {
            Some(token) if !matches!(token.kind, TokenKind::StreamStart) => {
                self.line_no == token.end_mark.line + 1
                    // sometimes token end marks are on the next line
                    && (token.end_mark.pointer == 0
                        || buffer[token.end_mark.pointer - 1] != '\n')
            }
            _ => false,
        }
    }
}

/// A token together with its neighbors, as passed to token rules.
#[derive(Debug, Clone)]
pub struct TokenElem {
    pub line_no: usize,
    pub curr: Token,
    pub prev: Option<Token>,
    pub next: Option<Token>,
    pub nextnext: Option<Token>,
}

// Token elements dominate the stream, so boxing them to please
// `large_enum_variant` would only add allocations.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum Elem {
    Token(TokenElem),
    Comment(Comment),
    Line(Line),
}

impl Elem {
    fn line_no(&self) -> usize {
        match self {
            Elem::Token(t) => t.line_no,
            Elem::Comment(c) => c.line_no,
            Elem::Line(l) => l.line_no,
        }
    }
}

pub fn line_generator(buffer: &[char]) -> Vec<Line> {
    // `buffer` includes the trailing '\0' sentinel.
    let raw_len = buffer.len().saturating_sub(1);
    let mut lines = Vec::new();
    let mut line_no = 1;
    let mut cur = 0;
    loop {
        let next = buffer[cur..raw_len].iter().position(|&c| c == '\n');
        match next {
            Some(offset) => {
                let next = cur + offset;
                if next > 0 && buffer[next - 1] == '\r' {
                    lines.push(Line {
                        line_no,
                        start: cur,
                        end: next - 1,
                    });
                } else {
                    lines.push(Line {
                        line_no,
                        start: cur,
                        end: next,
                    });
                }
                cur = next + 1;
                line_no += 1;
            }
            None => {
                lines.push(Line {
                    line_no,
                    start: cur,
                    end: raw_len,
                });
                break;
            }
        }
    }
    lines
}

fn comments_between_tokens(
    token1: &Token,
    token2: Option<&Token>,
    buffer: &[char],
    out: &mut Vec<Elem>,
) {
    let (buf_start, buf_end) = match token2 {
        None => (token1.end_mark.pointer, buffer.len()),
        Some(token2) => {
            if token1.end_mark.line == token2.start_mark.line
                && !matches!(token1.kind, TokenKind::StreamStart)
                && !matches!(token2.kind, TokenKind::StreamEnd)
            {
                return;
            }
            (token1.end_mark.pointer, token2.start_mark.pointer)
        }
    };

    let mut line_no = token1.end_mark.line + 1;
    let mut column_no = token1.end_mark.column + 1;
    let mut pointer = buf_start;

    let mut comment_before: Option<(usize, bool)> = None;
    let mut line_start = buf_start;
    loop {
        // Split the gap on '\n' (the final segment has no trailing break).
        let line_end = buffer[line_start..buf_end]
            .iter()
            .position(|&c| c == '\n')
            .map(|p| line_start + p)
            .unwrap_or(buf_end);
        let line = &buffer[line_start..line_end];
        if let Some(pos) = line.iter().position(|&c| c == '#') {
            let comment = Comment {
                line_no,
                column_no: column_no + pos,
                pointer: pointer + pos,
                token_before: Some(token1.clone()),
                token_after: token2.cloned(),
                comment_before,
            };
            let col = comment.column_no;
            let inline = comment.is_inline(buffer);
            out.push(Elem::Comment(comment));
            comment_before = Some((col, inline));
        }
        pointer += line.len() + 1;
        line_no += 1;
        column_no = 1;
        if line_end >= buf_end {
            break;
        }
        line_start = line_end + 1;
    }
}

/// Generates tokens (with neighbors) and the comments found between them.
/// Scanner errors silently end the stream.
pub fn token_or_comment_generator(content: &str) -> (Vec<Elem>, Vec<char>) {
    let mut scanner = Scanner::new(content);
    let mut elems: Vec<Elem> = Vec::new();

    let mut prev: Option<Token> = None;
    let mut curr: Option<Token> = scanner.get_token().unwrap_or_default();

    while let Some(curr_token) = curr {
        let next = match scanner.get_token() {
            Ok(t) => t,
            Err(_) => break,
        };
        let nextnext = match scanner.peek_token_cloned() {
            Ok(t) => t,
            Err(_) => break,
        };

        elems.push(Elem::Token(TokenElem {
            line_no: curr_token.start_mark.line + 1,
            curr: curr_token.clone(),
            prev: prev.take(),
            next: next.clone(),
            nextnext,
        }));

        comments_between_tokens(&curr_token, next.as_ref(), scanner.buffer(), &mut elems);

        prev = Some(curr_token);
        curr = next;
    }

    (elems, scanner.into_buffer())
}

/// Mixes tokens/comments and lines, ordered by line number (tokens first on
/// ties).
pub fn token_or_comment_or_line_generator(content: &str) -> (Vec<Elem>, Vec<char>) {
    let (tok_or_com, buffer) = token_or_comment_generator(content);
    let lines = line_generator(&buffer);

    // Consume the token/comment stream instead of cloning it: each element
    // owns up to four `Token`s, so a wholesale re-clone here is expensive.
    let mut merged = Vec::with_capacity(tok_or_com.len() + lines.len());
    let mut toks = tok_or_com.into_iter().peekable();
    let mut li = 0;
    loop {
        let take_line = match toks.peek() {
            None => li < lines.len(),
            Some(t) => li < lines.len() && t.line_no() > lines[li].line_no,
        };
        if take_line {
            merged.push(Elem::Line(lines[li]));
            li += 1;
        } else {
            match toks.next() {
                Some(t) => merged.push(t),
                None => break,
            }
        }
    }
    (merged, buffer)
}

// --- Disable directives ----------------------------------------------------

struct DisableDirective {
    rules: std::collections::HashSet<&'static str>,
    all_rules: Vec<&'static str>,
}

impl DisableDirective {
    fn new(all_rules: &[&'static str]) -> Self {
        Self {
            rules: Default::default(),
            all_rules: all_rules.to_vec(),
        }
    }

    /// Parse `rule:xxx` items after a directive prefix.
    fn parse_rules(rest: &str) -> Vec<String> {
        let items: Vec<&str> = rest.trim_end().split(' ').collect();
        items
            .iter()
            .map(|item| {
                let mut chars = item.chars();
                for _ in 0..5 {
                    chars.next();
                }
                chars.as_str().to_string()
            })
            .skip(1)
            .collect()
    }

    fn process_comment(&mut self, comment: &str) {
        if DISABLE_RULE.is_match(comment) {
            let rules = Self::parse_rules(&comment[18..]);
            if rules.is_empty() {
                self.rules = self.all_rules.iter().copied().collect();
            } else {
                for id in rules {
                    if let Some(&known) = self.all_rules.iter().find(|&&r| r == id) {
                        self.rules.insert(known);
                    }
                }
            }
        } else if ENABLE_RULE.is_match(comment) {
            let rules = Self::parse_rules(&comment[17..]);
            if rules.is_empty() {
                self.rules.clear();
            } else {
                for id in rules {
                    self.rules.retain(|&r| r != id);
                }
            }
        }
    }

    fn process_line_comment(&mut self, comment: &str) {
        if DISABLE_LINE_RULE.is_match(comment) {
            let rules = Self::parse_rules(&comment[23..]);
            if rules.is_empty() {
                self.rules = self.all_rules.iter().copied().collect();
            } else {
                for id in rules {
                    if let Some(&known) = self.all_rules.iter().find(|&&r| r == id) {
                        self.rules.insert(known);
                    }
                }
            }
        }
    }

    fn is_disabled(&self, problem: &LintProblem) -> bool {
        match problem.rule {
            Some(rule) => self.rules.contains(rule),
            None => false,
        }
    }
}

static DISABLE_RULE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^# yamllint disable( rule:\S+)*\s*$").unwrap());
static ENABLE_RULE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^# yamllint enable( rule:\S+)*\s*$").unwrap());
static DISABLE_LINE_RULE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"^# yamllint disable-line( rule:\S+)*\s*$").unwrap()
});
static DISABLE_FILE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"^#\s*yamllint disable-file\s*$").unwrap());

// --- Main entry points -------------------------------------------------------

fn get_cosmetic_problems(
    content: &str,
    conf: &YamlLintConfig,
    filepath: Option<&str>,
) -> Vec<LintProblem> {
    let enabled = conf.enabled_rules(filepath);
    let enabled_ids: Vec<&'static str> = enabled.iter().map(|r| r.id).collect();

    let mut runners: Vec<rules::RuleRunner> = enabled
        .iter()
        .map(|r| rules::RuleRunner::new(r.id, r.conf, r.level))
        .collect();

    let (elems, buffer) = token_or_comment_or_line_generator(content);

    let mut problems = Vec::new();
    let mut cache: Vec<LintProblem> = Vec::new();
    let mut disabled = DisableDirective::new(&enabled_ids);
    let mut disabled_for_line = DisableDirective::new(&enabled_ids);
    let mut disabled_for_next_line = DisableDirective::new(&enabled_ids);

    for elem in &elems {
        match elem {
            Elem::Token(token_elem) => {
                for runner in runners.iter_mut() {
                    runner.check_token(token_elem, &buffer, &mut cache);
                }
            }
            Elem::Comment(comment) => {
                for runner in runners.iter_mut() {
                    runner.check_comment(comment, &buffer, &mut cache);
                }

                let text = comment.text(&buffer);
                disabled.process_comment(&text);
                if comment.is_inline(&buffer) {
                    disabled_for_line.process_line_comment(&text);
                } else {
                    disabled_for_next_line.process_line_comment(&text);
                }
            }
            Elem::Line(line) => {
                for runner in runners.iter_mut() {
                    runner.check_line(line, &buffer, &mut cache);
                }

                // This is the last token/comment/line of this line; flush the
                // problems found, filtered by the directives.
                for problem in cache.drain(..) {
                    if !(disabled_for_line.is_disabled(&problem) || disabled.is_disabled(&problem))
                    {
                        problems.push(problem);
                    }
                }

                std::mem::swap(&mut disabled_for_line, &mut disabled_for_next_line);
                disabled_for_next_line = DisableDirective::new(&enabled_ids);
            }
        }
    }

    problems
}

fn get_syntax_error(content: &str) -> Option<LintProblem> {
    Parser::check_syntax(content).and_then(|e| {
        e.problem_mark.map(|mark| LintProblem {
            line: mark.line + 1,
            column: mark.column + 1,
            desc: format!("syntax error: {} (syntax)", e.problem),
            rule: None,
            level: Level::Error,
        })
    })
}

/// Lint a YAML source.
pub fn run(content: &str, conf: &YamlLintConfig, filepath: Option<&str>) -> Vec<LintProblem> {
    if let Some(path) = filepath
        && conf.is_file_ignored(path)
    {
        return Vec::new();
    }

    // First line: `# yamllint disable-file` skips the whole file.
    let first_line: String = content.chars().take_while(|&c| c != '\n').collect();
    let first_line = first_line.strip_suffix('\r').unwrap_or(&first_line);
    if DISABLE_FILE.is_match(first_line) {
        return Vec::new();
    }

    let mut syntax_error = get_syntax_error(content);

    let mut out = Vec::new();
    for problem in get_cosmetic_problems(content, conf, filepath) {
        // Insert the syntax error (if any) at the right place...
        if let Some(se) = &syntax_error
            && se.line <= problem.line
            && se.column <= problem.column
        {
            out.push(syntax_error.take().unwrap());
            // Discard the problem since it is at the same place as the
            // syntax error and is probably redundant.
            continue;
        }
        out.push(problem);
    }
    if let Some(se) = syntax_error {
        out.push(se);
    }
    out
}
