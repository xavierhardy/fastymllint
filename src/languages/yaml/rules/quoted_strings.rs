use crate::diagnostic::{Diagnostic, Fix, Location};
use crate::rule::{Rule, RuleContext};

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum QuoteType {
    Single,
    Double,
    Consistent,
    #[default]
    Any,
}

pub struct QuotedStrings {
    pub quote_type: QuoteType,
    pub required: String, // Can be "true", "false", "only-when-needed"
    pub extra_required: Vec<String>,
    pub extra_allowed: Vec<String>,
    pub allow_quoted_quotes: bool,
    pub check_keys: bool,
}

impl Default for QuotedStrings {
    fn default() -> Self {
        Self {
            quote_type: QuoteType::default(),
            required: "only-when-needed".to_string(),
            extra_required: Vec::new(),
            extra_allowed: Vec::new(),
            allow_quoted_quotes: false,
            check_keys: false,
        }
    }
}

impl Rule for QuotedStrings {
    fn name(&self) -> &'static str {
        "quoted-strings"
    }

    fn description(&self) -> &'static str {
        "Enforce string quoting rules"
    }

    #[allow(unused_variables)]
    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let quote_type = config
            .and_then(|c| c.get_option("quote-type"))
            .unwrap_or(self.quote_type);
        let required = config
            .and_then(|c| c.get_string("required"))
            .unwrap_or_else(|| self.required.clone());
        let extra_required = config
            .and_then(|c| c.get_option("extra-required"))
            .unwrap_or_else(|| self.extra_required.clone());
        let extra_allowed = config
            .and_then(|c| c.get_option("extra-allowed"))
            .unwrap_or_else(|| self.extra_allowed.clone());
        let allow_quoted_quotes = config
            .and_then(|c| c.get_option("allow-quoted-quotes"))
            .unwrap_or(self.allow_quoted_quotes);
        let check_keys = config
            .and_then(|c| c.get_option("check-keys"))
            .unwrap_or(self.check_keys);

        let mut diagnostics = Vec::new();

        let mut detected_quote_type: Option<char> = None;

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Simple check for now, needs proper YAML parsing for full accuracy
            let mut parts = line.split(':');
            let key_part = parts.next().unwrap_or("").trim();
            let value_part = parts.next().unwrap_or("").trim();

            let mut check_string = |s: &str, _is_key: bool| -> Option<Diagnostic> {
                if s.is_empty() {
                    return None;
                }

                let starts_with_quote = s.starts_with('\'') || s.starts_with('"');
                let ends_with_quote = s.ends_with('\'') || s.ends_with('"');
                let is_quoted = starts_with_quote && ends_with_quote;

                if required == "true" && !is_quoted {
                    return Some(
                        Diagnostic::error(
                            self.name(),
                            "string must be quoted",
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                        )
                        .with_fix(Fix::new(
                            "add quotes",
                            format!("'{}'", s),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1 + s.len()),
                        )),
                    );
                } else if required == "false" && is_quoted {
                    return Some(
                        Diagnostic::error(
                            self.name(),
                            "string must not be quoted",
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                        )
                        .with_fix(Fix::new(
                            "remove quotes",
                            s[1..s.len() - 1].to_string(),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1 + s.len()),
                        )),
                    );
                } else if required == "only-when-needed"
                    && !is_quoted
                    && (s.contains(' ')
                        || s.contains(':')
                        || s.contains('[')
                        || s.contains('{')
                        || s.contains(']')
                        || s.contains('}'))
                {
                    // This is a very simplistic "when needed" check
                    return Some(
                        Diagnostic::warning(
                            self.name(),
                            "string might need quotes",
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                        )
                        .with_fix(Fix::new(
                            "add quotes",
                            format!("'{}'", s),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                            Location::new(line_num, line.find(s).unwrap_or(0) + 1 + s.len()),
                        )),
                    );
                }

                // Check specific quote type if quoted
                if is_quoted {
                    let current_quote = s.chars().next().unwrap();
                    match quote_type {
                        QuoteType::Single => {
                            if current_quote != '\'' {
                                return Some(
                                    Diagnostic::error(
                                        self.name(),
                                        "string must use single quotes",
                                        Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                                    )
                                    .with_fix(Fix::new(
                                        "change to single quotes",
                                        format!("'{}'", &s[1..s.len() - 1]),
                                        Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                                        Location::new(
                                            line_num,
                                            line.find(s).unwrap_or(0) + 1 + s.len(),
                                        ),
                                    )),
                                );
                            }
                        }
                        QuoteType::Double => {
                            if current_quote != '"' {
                                return Some(
                                    Diagnostic::error(
                                        self.name(),
                                        "string must use double quotes",
                                        Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                                    )
                                    .with_fix(Fix::new(
                                        "change to double quotes",
                                        format!("\"{}\"", &s[1..s.len() - 1]),
                                        Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                                        Location::new(
                                            line_num,
                                            line.find(s).unwrap_or(0) + 1 + s.len(),
                                        ),
                                    )),
                                );
                            }
                        }
                        QuoteType::Consistent => {
                            if let Some(expected_quote) = detected_quote_type {
                                if expected_quote != current_quote {
                                    return Some(Diagnostic::error(
                                        self.name(),
                                        format!(
                                            "inconsistent quote type: expected '{}'",
                                            expected_quote
                                        ),
                                        Location::new(line_num, line.find(s).unwrap_or(0) + 1),
                                    ));
                                }
                            } else {
                                detected_quote_type = Some(current_quote);
                            }
                        }
                        QuoteType::Any => {}
                    }
                }

                None
            };

            if check_keys && let Some(diag) = check_string(key_part, true) {
                diagnostics.push(diag);
            }
            if let Some(diag) = check_string(value_part, false) {
                diagnostics.push(diag);
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        true
    }
}
