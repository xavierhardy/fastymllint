use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

#[derive(Default)] // Add derive(Default) as per clippy suggestion
pub struct EmptyValues {
    pub forbid_in_block_mappings: bool,
    pub forbid_in_flow_mappings: bool,
    pub forbid_in_block_sequences: bool,
}

impl Rule for EmptyValues {
    fn name(&self) -> &'static str {
        "empty-values"
    }

    fn description(&self) -> &'static str {
        "Forbid empty values"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let forbid_in_block_mappings = config
            .and_then(|c| c.get_option("forbid-in-block-mappings"))
            .unwrap_or(self.forbid_in_block_mappings);
        let _forbid_in_flow_mappings = config
            .and_then(|c| c.get_option("forbid-in-flow-mappings"))
            .unwrap_or(self.forbid_in_flow_mappings);
        let forbid_in_block_sequences = config
            .and_then(|c| c.get_option("forbid-in-block-sequences"))
            .unwrap_or(self.forbid_in_block_sequences);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();
            let current_indent = line.len() - line.trim_start().len();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_pos_trimmed) = trimmed.find(':') {
                let actual_colon_pos = line.find(':').unwrap_or(0);

                // Check if the value part on the same line is empty
                let value_on_same_line_trimmed = trimmed[colon_pos_trimmed + 1..].trim();

                if value_on_same_line_trimmed.is_empty() {
                    // It's an empty value on the same line

                    // Check if it's a block mapping header (e.g., "key:")
                    // This is heuristic: if next line is more indented, it's a block structure
                    let is_block_mapping_header = if i + 1 < ctx.lines.len() {
                        let next_line = ctx.lines[i + 1];
                        let next_trimmed = next_line.trim_start();
                        if !next_trimmed.is_empty() && !next_trimmed.starts_with('#') {
                            next_line.len() - next_trimmed.len() > current_indent
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Check for flow mapping (e.g., "key: {}") - not strictly "empty" by yamllint
                    let is_flow_mapping_empty =
                        value_on_same_line_trimmed == "{}" || value_on_same_line_trimmed == "[]";

                    if !is_block_mapping_header && !is_flow_mapping_empty {
                        // This is a truly empty scalar value
                        if forbid_in_block_mappings {
                            // Applies to both block and flow here unless further differentiated
                            diagnostics.push(Diagnostic::error(
                                self.name(),
                                "empty value in mapping",
                                Location::new(line_num, actual_colon_pos + 2),
                            ));
                        }
                    }
                }
            }

            if forbid_in_block_sequences && trimmed == "-" {
                diagnostics.push(Diagnostic::error(
                    self.name(),
                    "empty value in block sequence",
                    Location::new(line_num, line.find('-').unwrap_or(0) + 2),
                ));
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
