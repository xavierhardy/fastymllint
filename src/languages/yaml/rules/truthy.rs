use crate::diagnostic::Diagnostic;
use crate::rule::{Rule, RuleContext};

pub struct Truthy {
    pub allowed_values: Vec<String>,
    pub check_keys: bool,
}

impl Default for Truthy {
    fn default() -> Self {
        Self {
            allowed_values: vec!["true".to_string(), "false".to_string()],
            check_keys: true,
        }
    }
}

impl Rule for Truthy {
    fn name(&self) -> &'static str {
        "truthy"
    }

    fn description(&self) -> &'static str {
        "Enforce truthy values"
    }

    fn check(
        &self,
        ctx: &RuleContext,
        config: Option<&crate::config::RuleConfig>,
    ) -> Vec<Diagnostic> {
        let allowed_values = config
            .and_then(|c| c.get_option("allowed-values"))
            .unwrap_or_else(|| self.allowed_values.clone());
        let check_keys = config
            .and_then(|c| c.get_option("check-keys"))
            .unwrap_or(self.check_keys);

        let mut diagnostics = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let line_num = i + 1;
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split(':').collect();
            let key = parts[0].trim();
            let value = parts.get(1).map(|v| v.trim()).unwrap_or("");

            if check_keys
                && (key == "yes" || key == "no" || key == "on" || key == "off")
                && !allowed_values.contains(&key.to_string())
            {
                diagnostics.push(Diagnostic::error(
                    self.name(),
                    format!("unsupported truthy value '{}'", key),
                    crate::diagnostic::Location::new(line_num, line.find(key).unwrap_or(0) + 1),
                ));
            }

            if (value == "yes" || value == "no" || value == "on" || value == "off")
                && !allowed_values.contains(&value.to_string())
            {
                diagnostics.push(Diagnostic::error(
                    self.name(),
                    format!("unsupported truthy value '{}'", value),
                    crate::diagnostic::Location::new(line_num, line.find(value).unwrap_or(0) + 1),
                ));
            }
        }
        diagnostics
    }

    fn is_fixable(&self) -> bool {
        false
    }
}
