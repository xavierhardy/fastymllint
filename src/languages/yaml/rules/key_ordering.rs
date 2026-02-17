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

    fn check(&self, ctx: &RuleContext, config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        let ignored_keys = config.and_then(|c| c.get_option("ignored-keys")).unwrap_or_else(|| self.ignored_keys.clone());

        let mut diagnostics = Vec::new();
        let mut key_stacks: HashMap<usize, Vec<String>> = HashMap::new();
        let mut prev_indent = 0;

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim_start();
            let current_indent = line.len() - trimmed.len();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            if let Some(colon_pos) = trimmed.find(':') {
                let key = trimmed[..colon_pos].trim().to_string();

                if ignored_keys.contains(&key) {
                    continue;
                }

                if current_indent < prev_indent {
                    key_stacks.retain(|&indent, _| indent <= current_indent);
                }
                
                let keys = key_stacks.entry(current_indent).or_default();

                if let Some(last_key) = keys.last() {
                    if &key < last_key {
                        diagnostics.push(Diagnostic::error(
                            self.name(),
                            format!("key '{}' is not in alphabetical order", key),
                            crate::diagnostic::Location::new(line_num, current_indent + 1),
                        ));
                    }
                }
                keys.push(key);
            }
            prev_indent = current_indent;
        }

        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
