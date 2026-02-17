//! YAML lint rules

mod anchors;
mod braces;
mod brackets;
mod colons;
mod commas;
mod comments;
mod comments_indentation;
mod document_markers;
mod empty_lines;
mod empty_values;
mod float_values;
mod hyphens;
mod indentation;
mod key_duplicates;
mod key_ordering;
mod line_length;
mod new_line_at_end;
mod new_lines;
mod octal_values;
mod quoted_strings;
mod trailing_spaces;
mod truthy;

use anyhow::{Context, Result};

use crate::config::LanguageConfig;
use crate::diagnostic::Diagnostic;
use crate::rule::{BoxedRule, RuleContext};

pub use anchors::Anchors;
pub use braces::Braces;
pub use brackets::Brackets;
pub use colons::Colons;
pub use commas::Commas;
pub use comments::Comments;
pub use comments_indentation::CommentsIndentation;
pub use document_markers::{DocumentEnd, DocumentStart, fix_document_end, fix_document_start};
pub use empty_lines::{EmptyLines, fix_empty_lines};
pub use empty_values::EmptyValues;
pub use float_values::FloatValues;
pub use hyphens::Hyphens;
pub use indentation::Indentation;
pub use key_duplicates::KeyDuplicates;
pub use key_ordering::KeyOrdering;
pub use line_length::LineLength;
pub use new_line_at_end::{NewLineAtEndOfFile, fix_new_line_at_end};
pub use new_lines::NewLines;
pub use octal_values::OctalValues;
pub use quoted_strings::QuotedStrings;
pub use trailing_spaces::{TrailingSpaces, fix_trailing_spaces};
pub use truthy::Truthy;

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
            .filter_map(|rule| {
                let rule_config = config.and_then(|c| c.rules.get(rule.name()));
                if rule_config.map(|rc| rc.is_enabled()).unwrap_or(true) {
                    Some((rule, rule_config))
                } else {
                    None
                }
            })
            .flat_map(|(rule, rule_config)| rule.check(ctx, rule_config))
            .collect()
    }

    /// Apply fixes from all fixable rules
    pub fn fix(&self, content: &str, config: Option<&LanguageConfig>) -> Result<String> {
        let mut result = content.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        loop {
            let ctx = RuleContext::new(&result);
            let diagnostics = self.check(&ctx, config);

            // Find the first diagnostic that has a fix
            let fix = diagnostics.iter().find_map(|d| d.fix.as_ref());

            if let Some(fix) = fix {
                let start_offset = ctx
                    .offset(fix.start)
                    .context("Invalid fix start location")?;
                let end_offset = ctx.offset(fix.end).context("Invalid fix end location")?;

                let mut new_content = String::with_capacity(result.len() + fix.replacement.len());
                new_content.push_str(&result[..start_offset]);
                new_content.push_str(&fix.replacement);
                new_content.push_str(&result[end_offset..]);

                result = new_content;
                iterations += 1;

                if iterations >= MAX_ITERATIONS {
                    break;
                }
            } else {
                // No more fixes to apply
                break;
            }
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
            Box::new(KeyDuplicates::default()),
            Box::new(Braces::default()),
            Box::new(Brackets::default()),
            Box::new(Colons::default()),
            Box::new(Commas::default()),
            Box::new(Comments::default()),
            Box::new(CommentsIndentation),
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
