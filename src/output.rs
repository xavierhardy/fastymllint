//! Output formats. All of yamllint's formats are supported with
//! byte-identical output — `parsable`, `standard`, `colored`, `github` and
//! `auto` (the default, resolved like yamllint: `github` inside GitHub
//! Actions, `colored` on a tty, `standard` otherwise) — plus two
//! fastymllint extensions: `text` (same as `parsable`) and `json`.

use std::io::IsTerminal;

use serde::Serialize;

use crate::linter::{Level, LintProblem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
    Yamllint,
    Colored,
    Github,
    Auto,
}

impl OutputFormat {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "text" | "parsable" => Some(OutputFormat::Text),
            "json" => Some(OutputFormat::Json),
            "yamllint" | "standard" => Some(OutputFormat::Yamllint),
            "colored" => Some(OutputFormat::Colored),
            "github" => Some(OutputFormat::Github),
            "auto" => Some(OutputFormat::Auto),
            _ => None,
        }
    }

    /// Resolve `auto` the way yamllint does: `github` when running inside a
    /// GitHub workflow, `colored` when stdout is a terminal that supports
    /// color, `standard` otherwise.
    pub fn resolve(self) -> Self {
        match self {
            OutputFormat::Auto => {
                if std::env::var_os("GITHUB_ACTIONS").is_some()
                    && std::env::var_os("GITHUB_WORKFLOW").is_some()
                {
                    OutputFormat::Github
                } else if supports_color() {
                    OutputFormat::Colored
                } else {
                    OutputFormat::Yamllint
                }
            }
            other => other,
        }
    }
}

fn supports_color() -> bool {
    let supported_platform = !(cfg!(windows)
        && !(std::env::var_os("ANSICON").is_some()
            || std::env::var_os("TERM").is_some_and(|t| t == "ANSI")));
    supported_platform && std::io::stdout().is_terminal()
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

/// Render yamllint's `colored` format: the standard format with the same
/// ANSI escapes and padding (yamllint pads on the escaped string, so the
/// thresholds include the escape characters).
fn format_colored(report: &FileReport, no_warnings: bool, out: &mut String) {
    let mut first = true;
    for problem in report.problems {
        if no_warnings && problem.level != Level::Error {
            continue;
        }
        if first {
            out.push_str(&format!("\x1b[4m{}\x1b[0m\n", report.path));
            first = false;
        }
        let mut line = format!("  \x1b[2m{}:{}\x1b[0m", problem.line, problem.column);
        while line.chars().count() < 20 {
            line.push(' ');
        }
        let color = if problem.level == Level::Warning {
            "\x1b[33m"
        } else {
            "\x1b[31m"
        };
        line.push_str(&format!("{color}{}\x1b[0m", problem.level.as_str()));
        while line.chars().count() < 38 {
            line.push(' ');
        }
        line.push_str(&problem.desc);
        if let Some(rule) = problem.rule {
            line.push_str(&format!("  \x1b[2m({rule})\x1b[0m"));
        }
        out.push_str(&line);
        out.push('\n');
    }
    if !first {
        out.push('\n');
    }
}

/// Render yamllint's `github` format: GitHub Actions workflow commands,
/// grouped per file.
fn format_github(report: &FileReport, no_warnings: bool, out: &mut String) {
    let mut first = true;
    for problem in report.problems {
        if no_warnings && problem.level != Level::Error {
            continue;
        }
        if first {
            out.push_str(&format!("::group::{}\n", report.path));
            first = false;
        }
        out.push_str(&format!(
            "::{} file={},line={},col={}::{}:{} ",
            problem.level.as_str(),
            report.path,
            problem.line,
            problem.column,
            problem.line,
            problem.column
        ));
        if let Some(rule) = problem.rule {
            out.push_str(&format!("[{rule}] "));
        }
        out.push_str(&problem.desc);
        out.push('\n');
    }
    if !first {
        out.push_str("::endgroup::\n\n");
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
        OutputFormat::Colored => {
            let mut out = String::new();
            for report in reports {
                format_colored(report, no_warnings, &mut out);
            }
            out
        }
        OutputFormat::Github => {
            let mut out = String::new();
            for report in reports {
                format_github(report, no_warnings, &mut out);
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
        // `auto` depends on the environment; resolve and render as the
        // concrete format.
        OutputFormat::Auto => render(reports, format.resolve(), no_warnings),
    }
}
