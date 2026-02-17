use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct EmptyValues {
    pub forbid_in_block_mappings: bool,
    pub forbid_in_flow_mappings: bool,
    pub forbid_in_block_sequences: bool,
}

impl Default for EmptyValues {
    fn default() -> Self {
        Self {
            forbid_in_block_mappings: true,
            forbid_in_flow_mappings: true,
            forbid_in_block_sequences: true,
        }
    }
}

impl Rule for EmptyValues {
    fn name(&self) -> &'static str {
        "empty-values"
    }

    fn description(&self) -> &'static str {
        "Forbid empty values"
    }

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let forbid_in_block_mappings = config.and_then(|c| c.get_option("forbid-in-block-mappings")).unwrap_or(self.forbid_in_block_mappings);
        let forbid_in_flow_mappings = config.and_then(|c| c.get_option("forbid-in-flow-mappings")).unwrap_or(self.forbid_in_flow_mappings);
        let forbid_in_block_sequences = config.and_then(|c| c.get_option("forbid-in-block-sequences")).unwrap_or(self.forbid_in_block_sequences);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if forbid_in_block_mappings {
                if let Some(colon_pos) = trimmed.find(':') {
                    if trimmed[colon_pos + 1..].trim().is_empty() {
                        let mut in_flow_mapping = false;
                        for c in line.chars() {
                            if c == '{' {
                                in_flow_mapping = true;
                                break;
                            }
                        }
                        if !in_flow_mapping {
                            diagnostics.push(Diagnostic::error(
                                self.name(),
                                "empty value in block mapping",
                                crate::diagnostic::Location::new(line_num, colon_pos + 2),
                            ));
                        }
                    }
                }
            }

            if forbid_in_flow_mappings {
                 if let Some(colon_pos) = trimmed.find(':') {
                    if trimmed[colon_pos + 1..].trim().is_empty() {
                        let mut in_flow_mapping = false;
                        for c in line.chars() {
                            if c == '{' {
                                in_flow_mapping = true;
                                break;
                            }
                        }
                        if in_flow_mapping {
                            diagnostics.push(Diagnostic::error(
                                self.name(),
                                "empty value in flow mapping",
                                crate::diagnostic::Location::new(line_num, colon_pos + 2),
                            ));
                        }
                    }
                }
            }

            if forbid_in_block_sequences {
                if trimmed.starts_with("- ") && trimmed.len() == 2 {
                     diagnostics.push(Diagnostic::error(
                        self.name(),
                        "empty value in block sequence",
                        crate::diagnostic::Location::new(line_num, 3),
                    ));
                }
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
