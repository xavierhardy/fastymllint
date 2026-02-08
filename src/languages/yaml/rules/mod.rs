//! YAML lint rules

mod trailing_spaces;
mod line_length;
mod new_line_at_end;
mod indentation;
mod document_markers;
mod empty_lines;
mod key_duplicates;
mod braces;
mod brackets;
mod colons;
mod commas;
mod comments;
mod comments_indentation;
mod empty_values;
mod float_values;
mod hyphens;
mod key_ordering;
mod new_lines;
mod octal_values;
mod quoted_strings;
mod truthy;
mod anchors;

use anyhow::Result;

use crate::config::LanguageConfig;
use crate::diagnostic::Diagnostic;
use crate::rule::{BoxedRule, RuleContext};

pub use trailing_spaces::{TrailingSpaces, fix_trailing_spaces};
pub use line_length::LineLength;
pub use new_line_at_end::{NewLineAtEndOfFile, fix_new_line_at_end};
pub use indentation::Indentation;
pub use document_markers::{DocumentStart, DocumentEnd, fix_document_start, fix_document_end};
pub use empty_lines::{EmptyLines, fix_empty_lines};
pub use key_duplicates::KeyDuplicates;
pub use braces::Braces;
pub use brackets::Brackets;
pub use colons::Colons;
pub use commas::Commas;
pub use comments::Comments;
pub use comments_indentation::CommentsIndentation;
pub use empty_values::EmptyValues;
pub use float_values::FloatValues;
pub use hyphens::Hyphens;
pub use key_ordering::KeyOrdering;
pub use new_lines::NewLines;
pub use octal_values::OctalValues;
pub use quoted_strings::QuotedStrings;
pub use truthy::Truthy;
pub use anchors::Anchors;

/// Collection of all YAML rules
pub struct RuleSet {
    rules: Vec<BoxedRule>,
}

impl RuleSet {
    pub fn new(rules: Vec<BoxedRule>) -> Self {
        Self { rules }
    }
    
    /// Check content against all enabled rules
    pub fn check(&self, ctx: &RuleContext, config: Option<&LanguageConfig>) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .filter(|rule| {
                config
                    .and_then(|c| c.rules.get(rule.name()))
                    .map(|rc| rc.is_enabled())
                    .unwrap_or(true)
            })
            .flat_map(|rule| rule.check(ctx))
            .collect()
    }
    
    /// Apply fixes from all fixable rules
    pub fn fix(&self, content: &str, config: Option<&LanguageConfig>) -> Result<String> {
        let mut result = content.to_string();
        
        // Apply trailing spaces fix
        if is_rule_enabled(config, "trailing-spaces") {
            result = trailing_spaces::fix_trailing_spaces(&result);
        }
        
        // Apply new line at end fix
        if is_rule_enabled(config, "new-line-at-end-of-file") {
            result = new_line_at_end::fix_new_line_at_end(&result);
        }
        
        // Apply empty lines fix
        if is_rule_enabled(config, "empty-lines") {
            let max = get_rule_option(config, "empty-lines", "max").unwrap_or(2);
            result = empty_lines::fix_empty_lines(&result, max);
        }
        
        // Apply document start fix
        if is_rule_enabled_explicit(config, "document-start") {
            result = document_markers::fix_document_start(&result);
        }
        
        // Apply document end fix
        if is_rule_enabled_explicit(config, "document-end") {
            result = document_markers::fix_document_end(&result);
        }
        
        Ok(result)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new(vec![
            Box::new(TrailingSpaces),
            Box::new(LineLength::default()),
            Box::new(NewLineAtEndOfFile),
            Box::new(Indentation::default()),
            Box::new(DocumentStart),
            Box::new(DocumentEnd),
            Box::new(EmptyLines::default()),
            Box::new(KeyDuplicates),
            Box::new(Braces::default()),
            Box::new(Brackets::default()),
            Box::new(Colons::default()),
            Box::new(Commas::default()),
            Box::new(Comments::default()),
            Box::new(CommentsIndentation::default()),
            Box::new(EmptyValues::default()),
            Box::new(FloatValues::default()),
            Box::new(Hyphens::default()),
            Box::new(KeyOrdering::default()),
            Box::new(NewLines::default()),
            Box::new(OctalValues::default()),
            Box::new(QuotedStrings::default()),
            Box::new(Truthy::default()),
            Box::new(Anchors::default()),
        ])
    }
}

fn is_rule_enabled(config: Option<&LanguageConfig>, rule: &str) -> bool {
    config
        .and_then(|c| c.rules.get(rule))
        .map(|rc| rc.is_enabled())
        .unwrap_or(true)
}

fn is_rule_enabled_explicit(config: Option<&LanguageConfig>, rule: &str) -> bool {
    config
        .and_then(|c| c.rules.get(rule))
        .map(|rc| rc.is_enabled())
        .unwrap_or(false) // Default to disabled for opt-in rules
}

fn get_rule_option<T: for<'de> serde::Deserialize<'de>>(
    config: Option<&LanguageConfig>,
    rule: &str,
    option: &str,
) -> Option<T> {
    config
        .and_then(|c| c.rules.get(rule))
        .and_then(|rc| rc.get_option(option))
}
