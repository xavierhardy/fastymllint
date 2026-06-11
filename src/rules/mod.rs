//! All lint rules, their configuration parsing and dispatch.

pub mod common;

pub mod anchors;
pub mod braces;
pub mod brackets;
pub mod colons;
pub mod commas;
pub mod comments;
pub mod comments_indentation;
pub mod document_end;
pub mod document_start;
pub mod empty_lines;
pub mod empty_values;
pub mod float_values;
pub mod hyphens;
pub mod indentation;
pub mod key_duplicates;
pub mod key_ordering;
pub mod line_length;
pub mod new_line_at_end_of_file;
pub mod new_lines;
pub mod octal_values;
pub mod quoted_strings;
pub mod trailing_spaces;
pub mod truthy;

use crate::pyyaml::value::{YamlMapping, YamlValue};
use regex::Regex;

use crate::linter::{Comment, Level, Line, LintProblem, TokenElem};

pub const ALL_RULE_IDS: &[&str] = &[
    "anchors",
    "braces",
    "brackets",
    "colons",
    "commas",
    "comments",
    "comments-indentation",
    "document-end",
    "document-start",
    "empty-lines",
    "empty-values",
    "float-values",
    "hyphens",
    "indentation",
    "key-duplicates",
    "key-ordering",
    "line-length",
    "new-line-at-end-of-file",
    "new-lines",
    "octal-values",
    "quoted-strings",
    "trailing-spaces",
    "truthy",
];

/// Returns the canonical `&'static str` for a rule name, if it exists.
pub fn rule_id(name: &str) -> Option<&'static str> {
    ALL_RULE_IDS.iter().copied().find(|&id| id == name)
}

/// Typed, validated rule options.
#[derive(Debug, Clone)]
pub enum RuleOptions {
    Anchors(anchors::Conf),
    Braces(braces::Conf),
    Brackets(brackets::Conf),
    Colons(colons::Conf),
    Commas(commas::Conf),
    Comments(comments::Conf),
    CommentsIndentation,
    DocumentEnd(document_end::Conf),
    DocumentStart(document_start::Conf),
    EmptyLines(empty_lines::Conf),
    EmptyValues(empty_values::Conf),
    FloatValues(float_values::Conf),
    Hyphens(hyphens::Conf),
    Indentation(indentation::Conf),
    KeyDuplicates(key_duplicates::Conf),
    KeyOrdering(key_ordering::Conf),
    LineLength(line_length::Conf),
    NewLineAtEndOfFile,
    NewLines(new_lines::Conf),
    OctalValues(octal_values::Conf),
    QuotedStrings(quoted_strings::Conf),
    TrailingSpaces,
    Truthy(truthy::Conf),
}

// --- Option extraction helpers ----------------------------------------------

fn opt_err(rule: &str, key: &str, expected: &str) -> String {
    format!("invalid config: option \"{key}\" of \"{rule}\" should be {expected}")
}

fn get_bool(rule: &str, key: &str, value: &YamlValue) -> Result<bool, String> {
    value.as_bool().ok_or_else(|| opt_err(rule, key, "bool"))
}

fn get_int(rule: &str, key: &str, value: &YamlValue) -> Result<i64, String> {
    if value.is_bool() {
        return Err(opt_err(rule, key, "int"));
    }
    value.as_i64().ok_or_else(|| opt_err(rule, key, "int"))
}

fn get_str_list(rule: &str, key: &str, value: &YamlValue) -> Result<Vec<String>, String> {
    let seq = value.as_sequence().ok_or_else(|| {
        format!(
            "invalid config: option \"{key}\" of \"{rule}\" should only contain values in [str]"
        )
    })?;
    seq.iter()
        .map(|v| {
            v.as_str().map(str::to_string).ok_or_else(|| {
                format!("invalid config: option \"{key}\" of \"{rule}\" should only contain values in [str]")
            })
        })
        .collect()
}

fn get_regex_list(rule: &str, key: &str, value: &YamlValue) -> Result<Vec<Regex>, String> {
    get_str_list(rule, key, value)?
        .into_iter()
        .map(|s| {
            Regex::new(&s)
                .map_err(|e| format!("invalid config: option \"{key}\" of \"{rule}\": {e}"))
        })
        .collect()
}

fn unknown_option(rule: &str, key: &str) -> String {
    format!("invalid config: unknown option \"{key}\" for rule \"{rule}\"")
}

/// Iterate over the non-generic (level/ignore) options of a rule mapping.
fn rule_options(map: &YamlMapping) -> impl Iterator<Item = (&str, &YamlValue)> {
    map.iter().filter_map(|(k, v)| {
        let key = k.as_str()?;
        if matches!(key, "level" | "ignore" | "ignore-from-file") {
            None
        } else {
            Some((key, v))
        }
    })
}

impl RuleOptions {
    /// Parse and validate the options mapping of a rule (defaults applied),
    /// with type checks and per-option error messages.
    pub fn parse(id: &str, map: &YamlMapping) -> Result<RuleOptions, String> {
        match id {
            "anchors" => {
                let mut conf = anchors::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "forbid-undeclared-aliases" => {
                            conf.forbid_undeclared_aliases = get_bool(id, key, value)?
                        }
                        "forbid-duplicated-anchors" => {
                            conf.forbid_duplicated_anchors = get_bool(id, key, value)?
                        }
                        "forbid-unused-anchors" => {
                            conf.forbid_unused_anchors = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Anchors(conf))
            }
            "braces" | "brackets" => {
                let mut forbid = braces::Forbid::No;
                let mut min_inside = 0;
                let mut max_inside = 0;
                let mut min_empty = -1;
                let mut max_empty = -1;
                for (key, value) in rule_options(map) {
                    match key {
                        "forbid" => {
                            forbid = match value {
                                YamlValue::Bool(true) => braces::Forbid::Yes,
                                YamlValue::Bool(false) => braces::Forbid::No,
                                YamlValue::Str(s) if s == "non-empty" => braces::Forbid::NonEmpty,
                                _ => {
                                    return Err(opt_err(
                                        id,
                                        key,
                                        "in (<class 'bool'>, 'non-empty')",
                                    ));
                                }
                            }
                        }
                        "min-spaces-inside" => min_inside = get_int(id, key, value)?,
                        "max-spaces-inside" => max_inside = get_int(id, key, value)?,
                        "min-spaces-inside-empty" => min_empty = get_int(id, key, value)?,
                        "max-spaces-inside-empty" => max_empty = get_int(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                if id == "braces" {
                    Ok(RuleOptions::Braces(braces::Conf {
                        forbid,
                        min_spaces_inside: min_inside,
                        max_spaces_inside: max_inside,
                        min_spaces_inside_empty: min_empty,
                        max_spaces_inside_empty: max_empty,
                    }))
                } else {
                    Ok(RuleOptions::Brackets(brackets::Conf {
                        forbid,
                        min_spaces_inside: min_inside,
                        max_spaces_inside: max_inside,
                        min_spaces_inside_empty: min_empty,
                        max_spaces_inside_empty: max_empty,
                    }))
                }
            }
            "colons" => {
                let mut conf = colons::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "max-spaces-before" => conf.max_spaces_before = get_int(id, key, value)?,
                        "max-spaces-after" => conf.max_spaces_after = get_int(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Colons(conf))
            }
            "commas" => {
                let mut conf = commas::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "max-spaces-before" => conf.max_spaces_before = get_int(id, key, value)?,
                        "min-spaces-after" => conf.min_spaces_after = get_int(id, key, value)?,
                        "max-spaces-after" => conf.max_spaces_after = get_int(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Commas(conf))
            }
            "comments" => {
                let mut conf = comments::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "require-starting-space" => {
                            conf.require_starting_space = get_bool(id, key, value)?
                        }
                        "ignore-shebangs" => conf.ignore_shebangs = get_bool(id, key, value)?,
                        "min-spaces-from-content" => {
                            conf.min_spaces_from_content = get_int(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Comments(conf))
            }
            "comments-indentation" => {
                if let Some((key, _)) = rule_options(map).next() {
                    return Err(unknown_option(id, key));
                }
                Ok(RuleOptions::CommentsIndentation)
            }
            "document-end" => {
                let mut conf = document_end::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "present" => conf.present = get_bool(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::DocumentEnd(conf))
            }
            "document-start" => {
                let mut conf = document_start::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "present" => conf.present = get_bool(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::DocumentStart(conf))
            }
            "empty-lines" => {
                let mut conf = empty_lines::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "max" => conf.max = get_int(id, key, value)?,
                        "max-start" => conf.max_start = get_int(id, key, value)?,
                        "max-end" => conf.max_end = get_int(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::EmptyLines(conf))
            }
            "empty-values" => {
                let mut conf = empty_values::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "forbid-in-block-mappings" => {
                            conf.forbid_in_block_mappings = get_bool(id, key, value)?
                        }
                        "forbid-in-flow-mappings" => {
                            conf.forbid_in_flow_mappings = get_bool(id, key, value)?
                        }
                        "forbid-in-block-sequences" => {
                            conf.forbid_in_block_sequences = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::EmptyValues(conf))
            }
            "float-values" => {
                let mut conf = float_values::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "require-numeral-before-decimal" => {
                            conf.require_numeral_before_decimal = get_bool(id, key, value)?
                        }
                        "forbid-scientific-notation" => {
                            conf.forbid_scientific_notation = get_bool(id, key, value)?
                        }
                        "forbid-nan" => conf.forbid_nan = get_bool(id, key, value)?,
                        "forbid-inf" => conf.forbid_inf = get_bool(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::FloatValues(conf))
            }
            "hyphens" => {
                let mut conf = hyphens::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "max-spaces-after" => conf.max_spaces_after = get_int(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Hyphens(conf))
            }
            "indentation" => {
                let mut conf = indentation::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "spaces" => {
                            conf.spaces = match value {
                                YamlValue::Str(s) if s == "consistent" => {
                                    indentation::Spaces::Consistent
                                }
                                v if v.as_i64().is_some() && !v.is_bool() => {
                                    indentation::Spaces::Fixed(v.as_i64().unwrap())
                                }
                                _ => {
                                    return Err(opt_err(
                                        id,
                                        key,
                                        "in (<class 'int'>, 'consistent')",
                                    ));
                                }
                            }
                        }
                        "indent-sequences" => {
                            conf.indent_sequences = match value {
                                YamlValue::Bool(true) => indentation::IndentSequences::True,
                                YamlValue::Bool(false) => indentation::IndentSequences::False,
                                YamlValue::Str(s) if s == "whatever" => {
                                    indentation::IndentSequences::Whatever
                                }
                                YamlValue::Str(s) if s == "consistent" => {
                                    indentation::IndentSequences::Consistent
                                }
                                _ => {
                                    return Err(opt_err(
                                        id,
                                        key,
                                        "in (<class 'bool'>, 'whatever', 'consistent')",
                                    ));
                                }
                            }
                        }
                        "check-multi-line-strings" => {
                            conf.check_multi_line_strings = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Indentation(conf))
            }
            "key-duplicates" => {
                let mut conf = key_duplicates::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "forbid-duplicated-merge-keys" => {
                            conf.forbid_duplicated_merge_keys = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::KeyDuplicates(conf))
            }
            "key-ordering" => {
                let mut conf = key_ordering::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "ignored-keys" => conf.ignored_keys = get_regex_list(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::KeyOrdering(conf))
            }
            "line-length" => {
                let mut conf = line_length::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "max" => conf.max = get_int(id, key, value)?,
                        "allow-non-breakable-words" => {
                            conf.allow_non_breakable_words = get_bool(id, key, value)?
                        }
                        "allow-non-breakable-inline-mappings" => {
                            conf.allow_non_breakable_inline_mappings = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::LineLength(conf))
            }
            "new-line-at-end-of-file" => {
                if let Some((key, _)) = rule_options(map).next() {
                    return Err(unknown_option(id, key));
                }
                Ok(RuleOptions::NewLineAtEndOfFile)
            }
            "new-lines" => {
                let mut conf = new_lines::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "type" => {
                            conf.r#type = match value.as_str() {
                                Some("unix") => new_lines::NewLineType::Unix,
                                Some("dos") => new_lines::NewLineType::Dos,
                                Some("platform") => new_lines::NewLineType::Platform,
                                _ => {
                                    return Err(opt_err(id, key, "in ('unix', 'dos', 'platform')"));
                                }
                            }
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::NewLines(conf))
            }
            "octal-values" => {
                let mut conf = octal_values::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "forbid-implicit-octal" => {
                            conf.forbid_implicit_octal = get_bool(id, key, value)?
                        }
                        "forbid-explicit-octal" => {
                            conf.forbid_explicit_octal = get_bool(id, key, value)?
                        }
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::OctalValues(conf))
            }
            "quoted-strings" => {
                let mut conf = quoted_strings::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "quote-type" => {
                            conf.quote_type = match value.as_str() {
                                Some("any") => quoted_strings::QuoteType::Any,
                                Some("single") => quoted_strings::QuoteType::Single,
                                Some("double") => quoted_strings::QuoteType::Double,
                                Some("consistent") => quoted_strings::QuoteType::Consistent,
                                _ => {
                                    return Err(opt_err(
                                        id,
                                        key,
                                        "in ('any', 'single', 'double', 'consistent')",
                                    ));
                                }
                            }
                        }
                        "required" => {
                            conf.required = match value {
                                YamlValue::Bool(true) => quoted_strings::Required::True,
                                YamlValue::Bool(false) => quoted_strings::Required::False,
                                YamlValue::Str(s) if s == "only-when-needed" => {
                                    quoted_strings::Required::OnlyWhenNeeded
                                }
                                _ => {
                                    return Err(opt_err(
                                        id,
                                        key,
                                        "in (True, False, 'only-when-needed')",
                                    ));
                                }
                            }
                        }
                        "extra-required" => conf.extra_required = get_regex_list(id, key, value)?,
                        "extra-allowed" => conf.extra_allowed = get_regex_list(id, key, value)?,
                        "allow-quoted-quotes" => {
                            conf.allow_quoted_quotes = get_bool(id, key, value)?
                        }
                        "check-keys" => conf.check_keys = get_bool(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                // VALIDATE
                if conf.required == quoted_strings::Required::True && !conf.extra_allowed.is_empty()
                {
                    return Err(format!(
                        "invalid config: {id}: cannot use both \"required: true\" and \"extra-allowed\""
                    ));
                }
                if conf.required == quoted_strings::Required::True
                    && !conf.extra_required.is_empty()
                {
                    return Err(format!(
                        "invalid config: {id}: cannot use both \"required: true\" and \"extra-required\""
                    ));
                }
                if conf.required == quoted_strings::Required::False
                    && !conf.extra_allowed.is_empty()
                {
                    return Err(format!(
                        "invalid config: {id}: cannot use both \"required: false\" and \"extra-allowed\""
                    ));
                }
                Ok(RuleOptions::QuotedStrings(conf))
            }
            "trailing-spaces" => {
                if let Some((key, _)) = rule_options(map).next() {
                    return Err(unknown_option(id, key));
                }
                Ok(RuleOptions::TrailingSpaces)
            }
            "truthy" => {
                let mut conf = truthy::Conf::default();
                for (key, value) in rule_options(map) {
                    match key {
                        "allowed-values" => {
                            let values = get_str_list(id, key, value)?;
                            for v in &values {
                                if !truthy::TRUTHY_1_1.contains(&v.as_str()) {
                                    return Err(format!(
                                        "invalid config: option \"{key}\" of \"{id}\" should only contain values in the TRUTHY list"
                                    ));
                                }
                            }
                            conf.allowed_values = values;
                        }
                        "check-keys" => conf.check_keys = get_bool(id, key, value)?,
                        _ => return Err(unknown_option(id, key)),
                    }
                }
                Ok(RuleOptions::Truthy(conf))
            }
            _ => Err(format!("invalid config: no such rule: \"{id}\"")),
        }
    }
}

// --- Per-file rule runner ----------------------------------------------------

enum RuleState {
    None,
    Anchors(anchors::Context),
    Indentation(indentation::Context),
    KeyDuplicates(key_duplicates::Context),
    KeyOrdering(key_ordering::Context),
    QuotedStrings(quoted_strings::Context),
    Truthy(truthy::Context),
}

pub struct RuleRunner {
    pub id: &'static str,
    level: Level,
    options: RuleOptions,
    state: RuleState,
}

impl RuleRunner {
    pub fn new(id: &'static str, options: &RuleOptions, level: Level) -> Self {
        let state = match options {
            RuleOptions::Anchors(_) => RuleState::Anchors(Default::default()),
            RuleOptions::Indentation(_) => RuleState::Indentation(Default::default()),
            RuleOptions::KeyDuplicates(_) => RuleState::KeyDuplicates(Default::default()),
            RuleOptions::KeyOrdering(_) => RuleState::KeyOrdering(Default::default()),
            RuleOptions::QuotedStrings(_) => RuleState::QuotedStrings(Default::default()),
            RuleOptions::Truthy(_) => RuleState::Truthy(Default::default()),
            _ => RuleState::None,
        };
        Self {
            id,
            level,
            options: options.clone(),
            state,
        }
    }

    fn finish(&self, problems: &mut [LintProblem], start: usize) {
        for problem in problems.iter_mut().skip(start) {
            problem.rule = Some(self.id);
            problem.level = self.level;
        }
    }

    pub fn check_token(
        &mut self,
        elem: &TokenElem,
        buffer: &[char],
        problems: &mut Vec<LintProblem>,
    ) {
        let start = problems.len();
        match (&self.options, &mut self.state) {
            (RuleOptions::Anchors(conf), RuleState::Anchors(ctx)) => {
                anchors::check(conf, elem, ctx, problems)
            }
            (RuleOptions::Braces(conf), _) => braces::check(conf, elem, buffer, problems),
            (RuleOptions::Brackets(conf), _) => brackets::check(conf, elem, buffer, problems),
            (RuleOptions::Colons(conf), _) => colons::check(conf, elem, buffer, problems),
            (RuleOptions::Commas(conf), _) => commas::check(conf, elem, buffer, problems),
            (RuleOptions::DocumentEnd(conf), _) => document_end::check(conf, elem, problems),
            (RuleOptions::DocumentStart(conf), _) => document_start::check(conf, elem, problems),
            (RuleOptions::EmptyValues(conf), _) => empty_values::check(conf, elem, problems),
            (RuleOptions::FloatValues(conf), _) => float_values::check(conf, elem, problems),
            (RuleOptions::Hyphens(conf), _) => hyphens::check(conf, elem, problems),
            (RuleOptions::Indentation(conf), RuleState::Indentation(ctx)) => {
                indentation::check(conf, elem, buffer, ctx, problems)
            }
            (RuleOptions::KeyDuplicates(conf), RuleState::KeyDuplicates(ctx)) => {
                key_duplicates::check(conf, elem, ctx, problems)
            }
            (RuleOptions::KeyOrdering(conf), RuleState::KeyOrdering(ctx)) => {
                key_ordering::check(conf, elem, ctx, problems)
            }
            (RuleOptions::OctalValues(conf), _) => octal_values::check(conf, elem, problems),
            (RuleOptions::QuotedStrings(conf), RuleState::QuotedStrings(ctx)) => {
                quoted_strings::check(conf, elem, buffer, ctx, problems)
            }
            (RuleOptions::Truthy(conf), RuleState::Truthy(ctx)) => {
                truthy::check(conf, elem, ctx, problems)
            }
            _ => {}
        }
        self.finish(problems, start);
    }

    pub fn check_comment(
        &mut self,
        comment: &Comment,
        buffer: &[char],
        problems: &mut Vec<LintProblem>,
    ) {
        let start = problems.len();
        match &self.options {
            RuleOptions::Comments(conf) => comments::check(conf, comment, buffer, problems),
            RuleOptions::CommentsIndentation => {
                comments_indentation::check(comment, buffer, problems)
            }
            _ => {}
        }
        self.finish(problems, start);
    }

    pub fn check_line(&mut self, line: &Line, buffer: &[char], problems: &mut Vec<LintProblem>) {
        let start = problems.len();
        match &self.options {
            RuleOptions::EmptyLines(conf) => empty_lines::check(conf, line, buffer, problems),
            RuleOptions::LineLength(conf) => line_length::check(conf, line, buffer, problems),
            RuleOptions::NewLineAtEndOfFile => {
                new_line_at_end_of_file::check(line, buffer, problems)
            }
            RuleOptions::NewLines(conf) => new_lines::check(conf, line, buffer, problems),
            RuleOptions::TrailingSpaces => trailing_spaces::check(line, buffer, problems),
            _ => {}
        }
        self.finish(problems, start);
    }
}
