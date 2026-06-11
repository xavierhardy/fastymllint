//! Linter configuration: `extends` (default/relaxed presets or file paths),
//! per-rule levels, gitignore-style `ignore` / `yaml-files` patterns and
//! rule option validation.

use std::path::{Path, PathBuf};

use ignore::gitignore::{Gitignore, GitignoreBuilder};

use crate::decoder::auto_decode;
use crate::linter::Level;
use crate::pyyaml::loader;
use crate::pyyaml::value::{YamlMapping, YamlValue};
use crate::rules::{self, RuleOptions};

/// The built-in configuration presets.
const DEFAULT_CONF: &str = include_str!("conf/default.yaml");
const RELAXED_CONF: &str = include_str!("conf/relaxed.yaml");

#[derive(Debug)]
pub struct YamlLintConfigError(pub String);

impl std::fmt::Display for YamlLintConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for YamlLintConfigError {}

type Result<T> = std::result::Result<T, YamlLintConfigError>;

fn err<T>(msg: impl Into<String>) -> Result<T> {
    Err(YamlLintConfigError(msg.into()))
}

#[derive(Debug, Clone)]
enum RawRule {
    Disabled,
    Map(YamlMapping),
    Invalid,
}

#[derive(Debug)]
pub struct RuleSpec {
    pub level: Level,
    pub ignore: Option<Gitignore>,
    pub options: RuleOptions,
}

#[derive(Debug)]
enum RuleEntry {
    Disabled,
    Enabled(RuleSpec),
}

pub struct EnabledRule<'a> {
    pub id: &'static str,
    pub level: Level,
    pub conf: &'a RuleOptions,
}

#[derive(Debug)]
pub struct YamlLintConfig {
    rules: Vec<(String, RuleEntry)>,
    ignore: Option<Gitignore>,
    yaml_files: Gitignore,
    pub locale: Option<String>,
}

fn build_gitignore(lines: &[String]) -> Result<Gitignore> {
    let mut builder = GitignoreBuilder::new("");
    for line in lines {
        builder
            .add_line(None, line)
            .map_err(|e| YamlLintConfigError(format!("invalid config: {e}")))?;
    }
    builder
        .build()
        .map_err(|e| YamlLintConfigError(format!("invalid config: {e}")))
}

fn gitignore_matches(gi: &Gitignore, path: &str) -> bool {
    let path = path.strip_prefix("./").unwrap_or(path);
    gi.matched_path_or_any_parents(Path::new(path), false)
        .is_ignore()
}

/// Read gitignore-style patterns from an `ignore` value (a multi-line
/// string or a list of strings).
fn ignore_from_value(value: &YamlValue, what: &str) -> Result<Gitignore> {
    match value {
        YamlValue::Str(s) => {
            let lines: Vec<String> = s.lines().map(str::to_string).collect();
            build_gitignore(&lines)
        }
        YamlValue::Seq(seq) if seq.iter().all(|v| v.is_string()) => {
            let lines: Vec<String> = seq
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect();
            build_gitignore(&lines)
        }
        _ => err(format!(
            "invalid config: {what} should contain file patterns"
        )),
    }
}

/// Read gitignore-style patterns from the files named by an
/// `ignore-from-file` value.
fn ignore_from_file_value(value: &YamlValue, error_msg: &str) -> Result<Gitignore> {
    let files: Vec<String> = match value {
        YamlValue::Str(s) => vec![s.clone()],
        YamlValue::Seq(seq) if seq.iter().all(|v| v.is_string()) => seq
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .collect(),
        _ => return err(error_msg),
    };
    let mut lines = Vec::new();
    for file in files {
        let data = std::fs::read(&file).map_err(|e| YamlLintConfigError(format!("{e}")))?;
        let text =
            auto_decode(&data).map_err(|e| YamlLintConfigError(format!("invalid config: {e}")))?;
        lines.extend(text.lines().map(str::to_string));
    }
    build_gitignore(&lines)
}

/// Intermediate state while resolving `extends`.
struct RawConfig {
    rules: Vec<(String, RawRule)>,
    ignore: Option<Gitignore>,
    yaml_files: Option<Gitignore>,
    locale: Option<String>,
}

impl RawConfig {
    fn parse(content: &str) -> Result<RawConfig> {
        let value = match loader::load(content) {
            Ok(v) => v,
            Err(e) => return err(format!("invalid config: {}", e.problem)),
        };

        let YamlValue::Map(conf) = value else {
            return err("invalid config: not a mapping");
        };

        let mut rules: Vec<(String, RawRule)> = Vec::new();
        if let Some(rules_value) = conf.get("rules") {
            let YamlValue::Map(rules_map) = rules_value else {
                return err("invalid config: rules should be a mapping");
            };
            for (key, val) in rules_map.iter() {
                let Some(name) = key.as_str() else {
                    return err("invalid config: rules should be a mapping");
                };
                let raw = match val {
                    YamlValue::Str(s) if s == "enable" => RawRule::Map(YamlMapping::new()),
                    YamlValue::Str(s) if s == "disable" => RawRule::Disabled,
                    YamlValue::Bool(false) => RawRule::Disabled,
                    YamlValue::Map(m) => RawRule::Map(m.clone()),
                    _ => RawRule::Invalid,
                };
                rules.push((name.to_string(), raw));
            }
        }

        let mut config = RawConfig {
            rules,
            ignore: None,
            yaml_files: None,
            locale: None,
        };

        // Does this conf override another conf that we need to load?
        if let Some(extends) = conf.get("extends") {
            let Some(name) = extends.as_str() else {
                return err("invalid config: extends should be a string");
            };
            let base = RawConfig::load_extended(name)?;
            config.extend(base);
        }

        if conf.contains_key("ignore") && conf.contains_key("ignore-from-file") {
            return err("invalid config: ignore and ignore-from-file keys cannot be used together");
        } else if let Some(value) = conf.get("ignore-from-file") {
            config.ignore = Some(ignore_from_file_value(
                value,
                "invalid config: ignore-from-file should contain filename(s), either as a list or string",
            )?);
        } else if let Some(value) = conf.get("ignore") {
            config.ignore = Some(ignore_from_value(value, "ignore")?);
        }

        if let Some(value) = conf.get("yaml-files") {
            match value {
                YamlValue::Seq(seq) if seq.iter().all(|v| v.is_string()) => {
                    let lines: Vec<String> = seq
                        .iter()
                        .map(|v| v.as_str().unwrap_or_default().to_string())
                        .collect();
                    config.yaml_files = Some(build_gitignore(&lines)?);
                }
                _ => {
                    return err("invalid config: yaml-files should be a list of file patterns");
                }
            }
        }

        if let Some(value) = conf.get("locale") {
            let Some(locale) = value.as_str() else {
                return err("invalid config: locale should be a string");
            };
            config.locale = Some(locale.to_string());
        }

        Ok(config)
    }

    fn load_extended(name: &str) -> Result<RawConfig> {
        // A built-in preset...
        if !name.contains('/') {
            match name {
                "default" => return RawConfig::parse(DEFAULT_CONF),
                "relaxed" => return RawConfig::parse(RELAXED_CONF),
                _ => {}
            }
        }
        // ...or a configuration file on the filesystem.
        let data = std::fs::read(name).map_err(|e| YamlLintConfigError(format!("{e}")))?;
        let content =
            auto_decode(&data).map_err(|e| YamlLintConfigError(format!("invalid config: {e}")))?;
        RawConfig::parse(&content)
    }

    /// Merge `base` under `self`: rule mappings are merged key by key,
    /// other rule entries replace the base entry, and new rules are
    /// appended after the base rules.
    fn extend(&mut self, mut base: RawConfig) {
        for (name, rule) in std::mem::take(&mut self.rules) {
            let base_entry = base.rules.iter_mut().find(|(n, _)| *n == name);
            match (&rule, base_entry) {
                (RawRule::Map(own_map), Some((_, base_rule @ RawRule::Map(_)))) => {
                    if let RawRule::Map(base_map) = base_rule {
                        for (k, v) in own_map.iter() {
                            base_map.insert(k.clone(), v.clone());
                        }
                    }
                }
                (_, Some((_, base_rule))) => {
                    *base_rule = rule;
                }
                (_, None) => {
                    base.rules.push((name, rule));
                }
            }
        }
        self.rules = std::mem::take(&mut base.rules);

        if base.ignore.is_some() {
            self.ignore = base.ignore;
        }
    }

    fn validate(self) -> Result<YamlLintConfig> {
        let mut rules: Vec<(String, RuleEntry)> = Vec::new();
        for (name, raw) in self.rules {
            let Some(id) = rules::rule_id(&name) else {
                return err(format!("invalid config: no such rule: \"{name}\""));
            };
            let entry = match raw {
                RawRule::Disabled => RuleEntry::Disabled,
                RawRule::Invalid => {
                    return err(format!(
                        "invalid config: rule \"{id}\": should be either \"enable\", \"disable\" or a mapping"
                    ));
                }
                RawRule::Map(map) => {
                    let level = match map.get("level") {
                        None => Level::Error,
                        Some(YamlValue::Str(s)) if s == "error" => Level::Error,
                        Some(YamlValue::Str(s)) if s == "warning" => Level::Warning,
                        Some(_) => {
                            return err("invalid config: level should be \"error\" or \"warning\"");
                        }
                    };

                    let ignore = if let Some(value) = map.get("ignore-from-file") {
                        Some(ignore_from_file_value(
                            value,
                            "invalid config: ignore-from-file should contain valid filename(s), either as a list or string",
                        )?)
                    } else if let Some(value) = map.get("ignore") {
                        Some(ignore_from_value(value, "ignore")?)
                    } else {
                        None
                    };

                    let options = RuleOptions::parse(id, &map).map_err(YamlLintConfigError)?;
                    RuleEntry::Enabled(RuleSpec {
                        level,
                        ignore,
                        options,
                    })
                }
            };
            rules.push((name, entry));
        }

        let yaml_files = match self.yaml_files {
            Some(gi) => gi,
            None => build_gitignore(&[
                "*.yaml".to_string(),
                "*.yml".to_string(),
                ".yamllint".to_string(),
            ])?,
        };

        Ok(YamlLintConfig {
            rules,
            ignore: self.ignore,
            yaml_files,
            locale: self.locale,
        })
    }
}

impl YamlLintConfig {
    pub fn from_content(content: &str) -> Result<Self> {
        RawConfig::parse(content)?.validate()
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let data = std::fs::read(path).map_err(|e| YamlLintConfigError(format!("{e}")))?;
        let content =
            auto_decode(&data).map_err(|e| YamlLintConfigError(format!("invalid config: {e}")))?;
        Self::from_content(&content)
    }

    pub fn default_config() -> Self {
        Self::from_content("extends: default").expect("embedded default config is valid")
    }

    pub fn is_file_ignored(&self, filepath: &str) -> bool {
        self.ignore
            .as_ref()
            .map(|gi| gitignore_matches(gi, filepath))
            .unwrap_or(false)
    }

    pub fn is_yaml_file(&self, filepath: &str) -> bool {
        let basename = Path::new(filepath)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        gitignore_matches(&self.yaml_files, &basename)
    }

    pub fn enabled_rules(&self, filepath: Option<&str>) -> Vec<EnabledRule<'_>> {
        self.rules
            .iter()
            .filter_map(|(name, entry)| match entry {
                RuleEntry::Disabled => None,
                RuleEntry::Enabled(spec) => {
                    if let (Some(path), Some(gi)) = (filepath, spec.ignore.as_ref())
                        && gitignore_matches(gi, path)
                    {
                        return None;
                    }
                    Some(EnabledRule {
                        id: rules::rule_id(name).expect("validated rule id"),
                        level: spec.level,
                        conf: &spec.options,
                    })
                }
            })
            .collect()
    }
}

/// Find the project configuration file: `.yamllint`, `.yamllint.yaml` or
/// `.yamllint.yml` in the given directory or any parent, stopping at the
/// home directory or the filesystem root.
pub fn find_project_config_filepath(start: &Path) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let mut dir = start.to_path_buf();
    loop {
        for name in [".yamllint", ".yamllint.yaml", ".yamllint.yml"] {
            let path = dir.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
        let abs = std::path::absolute(&dir).unwrap_or_else(|_| dir.clone());
        if let Some(home) = &home
            && &abs == home
        {
            return None;
        }
        let parent = abs.parent()?.to_path_buf();
        if parent == abs {
            return None;
        }
        dir = parent;
    }
}
