use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NewLineType {
    Unix,
    Dos,
    Platform,
}

impl Default for NewLineType {
    fn default() -> Self {
        NewLineType::Unix
    }
}

pub struct NewLines {
    pub type_: NewLineType,
}

impl Default for NewLines {
    fn default() -> Self {
        Self {
            type_: NewLineType::default(),
        }
    }
}

impl Rule for NewLines {
    fn name(&self) -> &'static str {
        "new-lines"
    }

    fn description(&self) -> &'static str {
        "Enforce new line type"
    }

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let type_ = config.and_then(|c| c.get_option("type")).unwrap_or(self.type_);

        let expected_newline = match type_ {
            NewLineType::Unix => "\n",
            NewLineType::Dos => "\r\n",
            NewLineType::Platform => if cfg!(windows) { "\r\n" } else { "\n" },
        };

        let mut diagnostics = Vec::new();
        let mut pos = 0;
        while let Some(found) = ctx.content[pos..].find('\n') {
            let newline_pos = pos + found;
            let is_crlf = newline_pos > 0 && ctx.content.as_bytes()[newline_pos - 1] == b'\r';
            let actual_newline = if is_crlf { "\r\n" } else { "\n" };

            if actual_newline != expected_newline {
                let (line_num, col) = ctx.lines.iter().enumerate().find_map(|(i, line)| {
                    let line_start = ctx.content[..newline_pos].rfind('\n').map_or(0, |p| p + 1);
                    if newline_pos >= line_start && newline_pos < line_start + line.len() + actual_newline.len() {
                        Some((i + 1, newline_pos - line_start + 1))
                    } else {
                        None
                    }
                }).unwrap_or((ctx.lines.len(), ctx.lines.last().map_or(0, |l| l.len()) + 1));
                
                let diag = Diagnostic::error(
                    self.name(),
                    "wrong new line character",
                    Location::new(line_num, col),
                );
                diagnostics.push(diag.with_fix(Fix::new(
                    "change new line character",
                    expected_newline,
                    Location::new(line_num, col),
                    Location::new(line_num, col + actual_newline.len()),
                )));
            }
            pos = newline_pos + 1;
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        true
    }
}
