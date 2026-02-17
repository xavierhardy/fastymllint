use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

pub struct Indentation {
    pub spaces: String,
    pub indent_sequences: bool,
    pub check_multi_line_strings: bool,
}

impl Default for Indentation {
    fn default() -> Self {
        Self {
            spaces: "consistent".to_string(),
            indent_sequences: true,
            check_multi_line_strings: false,
        }
    }
}

impl Rule for Indentation {
    fn name(&self) -> &'static str {
        "indentation"
    }

    fn description(&self) -> &'static str {
        "Check for consistent indentation"
    }

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let spaces = config.and_then(|c| c.get_string("spaces")).unwrap_or_else(|| self.spaces.clone());
        let indent_sequences = config.and_then(|c| c.get_option("indent-sequences")).unwrap_or(self.indent_sequences);

        let mut diagnostics = Vec::new();
        let mut prev_indent = 0;
        let mut indent_size = if spaces == "consistent" { 0 } else { spaces.parse().unwrap_or(2) };
        let mut sequence_indent: Option<usize> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let current_indent = line.len() - trimmed.len();

            if trimmed.starts_with("- ") {
                if let Some(seq_indent) = sequence_indent {
                    if indent_sequences && current_indent != seq_indent {
                        diagnostics.push(Diagnostic::error(
                            self.name(),
                            "wrong indentation for sequence",
                            Location::new(line_num, 1),
                        ));
                    }
                } else {
                    sequence_indent = Some(current_indent);
                }
            } else {
                sequence_indent = None;
            }

            if current_indent > prev_indent {
                let diff = current_indent - prev_indent;
                if indent_size == 0 {
                    indent_size = diff;
                } else if diff != indent_size {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        format!("wrong indentation: expected {}, found {}", indent_size, diff),
                        Location::new(line_num, 1),
                    ));
                }
            } else if current_indent < prev_indent {
                let diff = prev_indent - current_indent;
                if indent_size != 0 && diff % indent_size != 0 {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        "wrong indentation: not a multiple of indent size",
                        Location::new(line_num, 1),
                    ));
                }
            }

            if indent_size != 0 && current_indent % indent_size != 0 {
                 diagnostics.push(Diagnostic::error(
                    self.name(),
                    "wrong indentation: not a multiple of indent size",
                    Location::new(line_num, 1),
                ));
            }

            prev_indent = current_indent;
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false // Too complex to fix reliably
    }
}


