//! Document start and end markers rules

use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

/// Rule that requires document start marker (---)
pub struct DocumentStart;

impl Rule for DocumentStart {
    fn name(&self) -> &'static str {
        "document-start"
    }

    fn description(&self) -> &'static str {
        "Require document start marker (---)"
    }

    fn check(&self, ctx: &RuleContext, _config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        if ctx.content.is_empty() {
            return vec![];
        }

        // Find first non-empty, non-comment line
        let first_content_line = ctx.lines.iter().enumerate().find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        });

        match first_content_line {
            Some((idx, line)) => {
                if line.trim() != "---" && !line.starts_with("---") {
                    vec![
                        Diagnostic::warning(
                            self.name(),
                            "missing document start \"---\"",
                            Location::new(idx + 1, 1),
                        )
                        .with_fix(Fix::insert(
                            "Add document start marker",
                            "---\n",
                            Location::new(1, 1),
                        )),
                    ]
                } else {
                    vec![]
                }
            }
            None => vec![], // Empty or comment-only file
        }
    }

    fn is_fixable(&self) -> bool {
        true
    }
}

/// Rule that requires document end marker (...)
pub struct DocumentEnd;

impl Rule for DocumentEnd {
    fn name(&self) -> &'static str {
        "document-end"
    }

    fn description(&self) -> &'static str {
        "Require document end marker (...)"
    }

    fn check(&self, ctx: &RuleContext, _config: Option<&crate::config::RuleConfig>) -> Vec<Diagnostic> {
        if ctx.content.is_empty() {
            return vec![];
        }

        // Find last non-empty, non-comment line
        let last_content_line = ctx.lines.iter().enumerate().rev().find(|(_, line)| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        });

        match last_content_line {
            Some((idx, line)) => {
                if line.trim() != "..." {
                    vec![
                        Diagnostic::warning(
                            self.name(),
                            "missing document end \"...\"",
                            Location::new(idx + 1, line.len() + 1),
                        )
                        .with_fix(Fix::insert(
                            "Add document end marker",
                            "\n...",
                            Location::new(idx + 1, line.len() + 1),
                        )),
                    ]
                } else {
                    vec![]
                }
            }
            None => vec![], // Empty file
        }
    }

    fn is_fixable(&self) -> bool {
        true
    }
}

/// Fix missing document start marker
pub fn fix_document_start(content: &str) -> String {
    if content.is_empty() {
        return content.to_string();
    }

    let lines: Vec<&str> = content.lines().collect();

    // Check if already has document start
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "---" || line.starts_with("---") {
            return content.to_string();
        }
        break;
    }

    format!("---\n{}", content)
}

/// Fix missing document end marker
pub fn fix_document_end(content: &str) -> String {
    if content.is_empty() {
        return content.to_string();
    }

    let content = content.trim_end();

    // Check if already has document end
    let lines: Vec<&str> = content.lines().collect();
    if let Some(last) = lines.last() {
        if last.trim() == "..." {
            return format!("{}\n", content);
        }
    }

    format!("{}\n...\n", content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_document_start() {
        let content = "key: value\n";
        let ctx = RuleContext::new(content);
        let diagnostics = DocumentStart.check(&ctx, None);

        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_has_document_start() {
        let content = "---\nkey: value\n";
        let ctx = RuleContext::new(content);
        let diagnostics = DocumentStart.check(&ctx, None);

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_fix_document_start() {
        assert_eq!(fix_document_start("key: value\n"), "---\nkey: value\n");
        assert_eq!(fix_document_start("---\nkey: value\n"), "---\nkey: value\n");
    }

    #[test]
    fn test_fix_document_end() {
        assert_eq!(fix_document_end("key: value"), "key: value\n...\n");
        assert_eq!(fix_document_end("key: value\n..."), "key: value\n...\n");
    }
}
