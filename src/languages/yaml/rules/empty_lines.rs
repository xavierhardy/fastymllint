//! Empty lines rule

use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that limits consecutive empty lines
pub struct EmptyLines {
    /// Maximum allowed consecutive empty lines
    max: usize,
    /// Maximum empty lines at start of file
    max_start: usize,
    /// Maximum empty lines at end of file
    max_end: usize,
}

impl EmptyLines {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            max_start: 0,
            max_end: 0,
        }
    }

    pub fn with_options(max: usize, max_start: usize, max_end: usize) -> Self {
        Self {
            max,
            max_start,
            max_end,
        }
    }
}

impl Default for EmptyLines {
    fn default() -> Self {
        Self::new(2)
    }
}

impl Rule for EmptyLines {
    fn name(&self) -> &'static str {
        "empty-lines"
    }

    fn description(&self) -> &'static str {
        "Limit the number of consecutive empty lines"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let max = config.and_then(|c| c.get_option("max")).unwrap_or(self.max);
        let max_start = config
            .and_then(|c| c.get_option("max-start"))
            .unwrap_or(self.max_start);
        let max_end = config
            .and_then(|c| c.get_option("max-end"))
            .unwrap_or(self.max_end);

        let mut diagnostics = Vec::new();
        let mut consecutive_empty = 0;
        let mut in_start = true;

        for (idx, line) in ctx.lines.iter().enumerate() {
            let line_num = idx + 1;

            if line.trim().is_empty() {
                consecutive_empty += 1;

                // Check start of file
                if in_start && consecutive_empty > max_start {
                    diagnostics.push(
                        Diagnostic::warning(
                            self.name(),
                            format!("too many blank lines at the start ({})", consecutive_empty),
                            Location::new(line_num, 1),
                        )
                        .with_fix(Fix::delete(
                            "remove blank line at the start",
                            Location::new(line_num, 1),
                            Location::new(line_num + 1, 1),
                        )),
                    );
                }
            } else {
                // Check consecutive empty lines in the middle
                if !in_start && consecutive_empty > max {
                    // Report for each extra line to allow fixing one by one
                    for i in (max + 1)..=consecutive_empty {
                        let report_line = line_num - (consecutive_empty - i + 1);
                        diagnostics.push(
                            Diagnostic::warning(
                                self.name(),
                                format!("too many blank lines ({} > {})", consecutive_empty, max),
                                Location::new(report_line, 1),
                            )
                            .with_fix(Fix::delete(
                                "remove extra blank line",
                                Location::new(report_line, 1),
                                Location::new(report_line + 1, 1),
                            )),
                        );
                    }
                }

                consecutive_empty = 0;
                in_start = false;
            }
        }

        // Check end of file
        if consecutive_empty > max_end {
            for i in (max_end + 1)..=consecutive_empty {
                let report_line = ctx.line_count() - (consecutive_empty - i);
                diagnostics.push(
                    Diagnostic::warning(
                        self.name(),
                        format!("too many blank lines at the end ({})", consecutive_empty),
                        Location::new(report_line, 1),
                    )
                    .with_fix(Fix::delete(
                        "remove blank line at the end",
                        Location::new(report_line, 1),
                        // If it's the very last line of the file, we might not have a next line
                        Location::new(report_line + 1, 1),
                    )),
                );
            }
        }

        diagnostics
    }

    fn is_fixable(&self) -> bool {
        true
    }
}

/// Fix excessive empty lines
pub fn fix_empty_lines(content: &str, max: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut consecutive_empty = 0;
    let mut started = false;

    for line in &lines {
        if line.trim().is_empty() {
            if started {
                consecutive_empty += 1;
                if consecutive_empty <= max {
                    result.push(*line);
                }
            }
            // Skip empty lines at start
        } else {
            started = true;
            consecutive_empty = 0;
            result.push(*line);
        }
    }

    // Remove trailing empty lines beyond max_end (0)
    while result.last().map(|l| l.trim().is_empty()).unwrap_or(false) {
        result.pop();
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') && !output.is_empty() {
        output.push('\n');
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_too_many_empty_lines() {
        let content = "hello:\n\n\n\n\nworld: value\n";
        let ctx = RuleContext::new(content);
        let rule = EmptyLines::new(2);
        let diagnostics = rule.check(&ctx, None);

        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_fix_empty_lines() {
        let content = "hello:\n\n\n\n\nworld: value\n";
        let fixed = fix_empty_lines(content, 2);
        assert_eq!(fixed, "hello:\n\n\nworld: value\n");
    }
}
