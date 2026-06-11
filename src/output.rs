//! Output formats: `text` (default), `json`, and `yamllint` (byte-identical
//! to the reference "standard" format).

use serde::Serialize;

use crate::linter::{Level, LintProblem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yamllint,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text" => Some(OutputFormat::Text),
            "json" => Some(OutputFormat::Json),
            "yamllint" | "standard" => Some(OutputFormat::Yamllint),
            _ => None,
        }
    }
}

/// One file's worth of problems.
pub struct FileReport<'a> {
    pub path: &'a str,
    pub problems: &'a [LintProblem],
}

#[derive(Serialize)]
struct JsonProblem<'a> {
    path: &'a str,
    line: usize,
    column: usize,
    level: &'a str,
    rule: Option<&'a str>,
    message: &'a str,
}

/// Render the `yamllint` output format for one file (the format yamllint
/// calls "standard": a file header line, aligned problem lines, and a
/// trailing blank line).
fn format_yamllint(report: &FileReport, no_warnings: bool, out: &mut String) {
    let mut first = true;
    for problem in report.problems {
        if no_warnings && problem.level != Level::Error {
            continue;
        }
        if first {
            out.push_str(report.path);
            out.push('\n');
            first = false;
        }
        let mut line = format!("  {}:{}", problem.line, problem.column);
        while line.chars().count() < 12 {
            line.push(' ');
        }
        line.push_str(problem.level.as_str());
        while line.chars().count() < 21 {
            line.push(' ');
        }
        line.push_str(&problem.desc);
        if let Some(rule) = problem.rule {
            line.push_str(&format!("  ({rule})"));
        }
        out.push_str(&line);
        out.push('\n');
    }
    if !first {
        out.push('\n');
    }
}

fn format_text(report: &FileReport, no_warnings: bool, out: &mut String) {
    for problem in report.problems {
        if no_warnings && problem.level != Level::Error {
            continue;
        }
        out.push_str(&format!(
            "{}:{}:{}: [{}] {}\n",
            report.path,
            problem.line,
            problem.column,
            problem.level.as_str(),
            problem.message()
        ));
    }
}

/// Render all reports in the chosen format. JSON is a single array over all
/// files; the other formats are concatenations of per-file output.
pub fn render(reports: &[FileReport], format: OutputFormat, no_warnings: bool) -> String {
    match format {
        OutputFormat::Json => {
            let mut items = Vec::new();
            for report in reports {
                for problem in report.problems {
                    if no_warnings && problem.level != Level::Error {
                        continue;
                    }
                    items.push(JsonProblem {
                        path: report.path,
                        line: problem.line,
                        column: problem.column,
                        level: problem.level.as_str(),
                        rule: problem.rule,
                        message: &problem.desc,
                    });
                }
            }
            let mut s = serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".into());
            s.push('\n');
            s
        }
        OutputFormat::Yamllint => {
            let mut out = String::new();
            for report in reports {
                format_yamllint(report, no_warnings, &mut out);
            }
            out
        }
        OutputFormat::Text => {
            let mut out = String::new();
            for report in reports {
                format_text(report, no_warnings, &mut out);
            }
            out
        }
    }
}
