use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};
use std::collections::HashMap;

#[derive(Default)]
pub struct KeyOrdering {
    pub ignored_keys: Vec<String>,
}

impl Rule for KeyOrdering {
    fn name(&self) -> &'static str {
        "key-ordering"
    }

    fn description(&self) -> &'static str {
        "Enforce alphabetical ordering of keys"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let ignored_keys = config
            .and_then(|c| c.get_option("ignored-keys"))
            .unwrap_or_else(|| self.ignored_keys.clone());

        let mut diagnostics = Vec::new();
        let mut key_stacks: HashMap<usize, Vec<String>> = HashMap::new();
        let _prev_indent = 0; // Marked as unused, will remove mut

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim_start();
            let current_indent = line.len() - trimmed.len();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(colon_pos) = trimmed.find(':') {
                // Ensure it's a key: value pair and not part of a block sequence or similar
                // A simple heuristic for now: check if it's not a list item
                if trimmed.starts_with('-') {
                    continue;
                }

                let key = trimmed[..colon_pos].trim().to_string();

                if ignored_keys.contains(&key) {
                    continue;
                }

                // If indentation decreased, remove keys from deeper levels
                key_stacks.retain(|&indent, _| indent <= current_indent);

                let keys_at_current_level = key_stacks.entry(current_indent).or_default();

                if let Some(last_key) = keys_at_current_level.last()
                    && key < *last_key
                {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        format!(
                            "key '{}' is not in alphabetical order (comes after '{}')",
                            key, last_key
                        ),
                        crate::diagnostic::Location::new(line_num, current_indent + 1),
                    ));
                }
                keys_at_current_level.push(key);
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
