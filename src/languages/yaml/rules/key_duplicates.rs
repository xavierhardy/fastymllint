use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that checks for duplicate keys in mappings
#[derive(Default)]
pub struct KeyDuplicates {
    pub allowed_keys: Vec<String>,
}

impl Rule for KeyDuplicates {
    fn name(&self) -> &'static str {
        "key-duplicates"
    }

    fn description(&self) -> &'static str {
        "Forbid duplicate keys in YAML mappings"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let allowed_keys = config
            .and_then(|c| c.get_option("allowed-keys"))
            .unwrap_or_else(|| self.allowed_keys.clone());

        let mut diagnostics = Vec::new();

        // Track keys at each indentation level
        // Map: indentation level -> (key -> first line number)
        let mut key_stacks: HashMap<usize, HashMap<String, usize>> = HashMap::new();
        let mut prev_indent = 0;

        for (idx, line) in ctx.lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let current_indent = line.len() - trimmed.len();

            // If we decreased indentation, clear keys at deeper levels
            if current_indent < prev_indent {
                key_stacks.retain(|&indent, _| indent <= current_indent);
            }

            // Check if this line defines a key
            if let Some(colon_pos) = find_key_colon(trimmed) {
                let key = trimmed[..colon_pos].trim().to_string();

                if allowed_keys.contains(&key) {
                    continue;
                }

                // Skip if key starts with special chars (anchors, etc.)
                if key.starts_with('&') || key.starts_with('*') || key.starts_with('-') {
                    prev_indent = current_indent;
                    continue;
                }

                // Get or create the key map for this indentation level
                let keys = key_stacks.entry(current_indent).or_default();

                if let Some(&first_line) = keys.get(&key) {
                    diagnostics.push(Diagnostic::error(
                        self.name(),
                        format!(
                            "duplication of key \"{}\" in mapping (first occurrence at line {})",
                            key, first_line
                        ),
                        Location::new(line_num, current_indent + 1),
                    ));
                } else {
                    keys.insert(key, line_num);
                }
            }

            prev_indent = current_indent;
        }

        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false // Cannot auto-fix duplicate keys (which one to keep?)
    }
}

/// Find the position of the key-value colon (not inside quotes or flow sequences)
fn find_key_colon(line: &str) -> Option<usize> {
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut depth: usize = 0;

    for (i, c) in line.char_indices() {
        match c {
            '"' | '\'' if !in_quote => {
                in_quote = true;
                quote_char = c;
            }
            c if c == quote_char && in_quote => {
                in_quote = false;
            }
            '[' | '{' if !in_quote => {
                depth += 1;
            }
            ']' | '}' if !in_quote => {
                depth = depth.saturating_sub(1);
            }
            ':' if !in_quote && depth == 0 => {
                // Check if this is a key colon (followed by space, newline, or end)
                let next = line.chars().nth(i + 1);
                if next.is_none() || next == Some(' ') || next == Some('\t') {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_keys() {
        let content = "name: foo\nvalue: 1\nname: bar\n";
        let ctx = RuleContext::new(content);
        let diagnostics = KeyDuplicates::default().check(&ctx, None);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].location.line, 3);
    }

    #[test]
    fn test_no_duplicates() {
        let content = "name: foo\nvalue: 1\nother: bar\n";
        let ctx = RuleContext::new(content);
        let diagnostics = KeyDuplicates::default().check(&ctx, None);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_nested_same_keys() {
        // Same key at different nesting levels is OK
        let content = "parent:\n  name: foo\nname: bar\n";
        let ctx = RuleContext::new(content);
        let diagnostics = KeyDuplicates::default().check(&ctx, None);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_find_key_colon() {
        assert_eq!(find_key_colon("key: value"), Some(3));
        assert_eq!(find_key_colon("\"key: with colon\": value"), Some(17));
        assert_eq!(find_key_colon("- list item"), None);
    }
}
