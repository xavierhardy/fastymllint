//! Pure-Rust integration tests (no Python required): linting behavior,
//! configuration handling, fixing and output formats.

use fastymllint::config::YamlLintConfig;
use fastymllint::fix::fix_content;
use fastymllint::linter::{self, Level};
use fastymllint::output::{FileReport, OutputFormat, render};

fn lint(content: &str) -> Vec<fastymllint::LintProblem> {
    let conf = YamlLintConfig::default_config();
    linter::run(content, &conf, None)
}

fn descs(problems: &[fastymllint::LintProblem]) -> Vec<String> {
    problems
        .iter()
        .map(|p| {
            format!(
                "{}:{} {} ({})",
                p.line,
                p.column,
                p.desc,
                p.rule.unwrap_or("syntax")
            )
        })
        .collect()
}

#[test]
fn clean_file_has_no_problems() {
    let problems = lint("---\nkey: value\nother:\n  - 1\n  - 2\n");
    assert!(problems.is_empty(), "{:?}", descs(&problems));
}

#[test]
fn detects_common_problems() {
    let problems = lint("key: value   \nother:  x\n");
    let d = descs(&problems);
    assert!(
        d.contains(&"1:1 missing document start \"---\" (document-start)".to_string()),
        "{d:?}"
    );
    assert!(
        d.contains(&"1:11 trailing spaces (trailing-spaces)".to_string()),
        "{d:?}"
    );
    assert!(
        d.contains(&"2:8 too many spaces after colon (colons)".to_string()),
        "{d:?}"
    );
}

#[test]
fn detects_syntax_error() {
    let problems = lint("---\nkey: value\n  bad: indent\n");
    assert!(
        problems.iter().any(|p| p.rule.is_none()
            && p.level == Level::Error
            && p.desc
                .starts_with("syntax error: mapping values are not allowed here")),
        "{:?}",
        descs(&problems)
    );
}

#[test]
fn disable_line_directive() {
    let problems = lint("---\na: 1   # yamllint disable-line rule:trailing-spaces   \nb: 2   \n");
    let d = descs(&problems);
    assert_eq!(d.len(), 1, "{d:?}");
    assert!(d[0].starts_with("3:5 trailing spaces"));
}

#[test]
fn disable_file_directive() {
    let problems = lint("# yamllint disable-file\nkey:   broken   \n");
    assert!(problems.is_empty());
}

#[test]
fn warning_vs_error_levels() {
    let problems = lint("key: value\n");
    // document-start is a warning in the default config.
    let doc_start = problems
        .iter()
        .find(|p| p.rule == Some("document-start"))
        .expect("document-start problem");
    assert_eq!(doc_start.level, Level::Warning);
}

#[test]
fn config_disable_rule() {
    let conf =
        YamlLintConfig::from_content("extends: default\nrules:\n  trailing-spaces: disable\n")
            .unwrap();
    let problems = linter::run("---\nkey: value   \n", &conf, None);
    assert!(problems.is_empty(), "{:?}", descs(&problems));
}

#[test]
fn config_rule_options() {
    let conf =
        YamlLintConfig::from_content("extends: default\nrules:\n  line-length:\n    max: 10\n")
            .unwrap();
    let problems = linter::run("---\nkey: a very long line here\n", &conf, None);
    assert!(
        problems
            .iter()
            .any(|p| p.desc == "line too long (26 > 10 characters)"),
        "{:?}",
        descs(&problems)
    );
}

#[test]
fn config_errors() {
    assert!(
        YamlLintConfig::from_content("rules:\n  no-such-rule: enable\n")
            .unwrap_err()
            .to_string()
            .contains("no such rule")
    );
    assert!(
        YamlLintConfig::from_content("rules:\n  colons:\n    bad-option: 1\n")
            .unwrap_err()
            .to_string()
            .contains("unknown option")
    );
    assert!(
        YamlLintConfig::from_content("not a mapping")
            .unwrap_err()
            .to_string()
            .contains("invalid config")
    );
}

#[test]
fn fix_safe_changes() {
    let conf = YamlLintConfig::default_config();
    let result = fix_content("key: value   \nother:  x", &conf, false);
    assert!(result.changed);
    assert_eq!(result.fixed, "---\nkey: value\nother: x\n");
    assert!(
        result.remaining.is_empty(),
        "{:?}",
        descs(&result.remaining)
    );
}

#[test]
fn fix_safe_does_not_touch_truthy() {
    let conf = YamlLintConfig::default_config();
    let safe = fix_content("---\nflag: yes\n", &conf, false);
    assert!(safe.fixed.contains("yes"));
    let unsafe_fix = fix_content("---\nflag: yes\n", &conf, true);
    assert_eq!(unsafe_fix.fixed, "---\nflag: true\n");
}

#[test]
fn fix_is_idempotent() {
    let conf = YamlLintConfig::default_config();
    let content = "key: value   \nlist:\n   - a\n   - b\nflag: yes  #bad comment\n";
    let once = fix_content(content, &conf, true);
    let twice = fix_content(&once.fixed, &conf, true);
    assert_eq!(once.fixed, twice.fixed);
    assert!(!twice.changed);
}

#[test]
fn yamllint_output_format() {
    let problems = lint("key: value   \n");
    let reports = [FileReport {
        path: "test.yaml",
        problems: &problems,
    }];
    let out = render(&reports, OutputFormat::Yamllint, false);
    assert!(out.starts_with("test.yaml\n"));
    assert!(
        out.contains("  1:1       warning  missing document start \"---\"  (document-start)\n"),
        "{out}"
    );
    assert!(out.ends_with("\n\n"));
}

#[test]
fn json_output_format() {
    let problems = lint("key: value   \n");
    let reports = [FileReport {
        path: "test.yaml",
        problems: &problems,
    }];
    let out = render(&reports, OutputFormat::Json, false);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid json");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), problems.len());
    assert_eq!(arr[0]["path"], "test.yaml");
    assert!(arr[0]["line"].is_number());
}

#[test]
fn colored_output_format() {
    let problems = lint("key: value   \n"); // warning (doc-start) + error (trailing)
    let reports = [FileReport {
        path: "test.yaml",
        problems: &problems,
    }];
    let out = render(&reports, OutputFormat::Colored, false);
    assert!(out.starts_with("\x1b[4mtest.yaml\x1b[0m\n"), "{out:?}");
    // Same layout as yamllint: padding thresholds count escape characters.
    assert!(
        out.contains(
            "  \x1b[2m1:1\x1b[0m       \x1b[33mwarning\x1b[0m  \
             missing document start \"---\"  \x1b[2m(document-start)\x1b[0m\n"
        ),
        "{out:?}"
    );
    assert!(out.contains("\x1b[31merror\x1b[0m"), "{out:?}");
    assert!(out.ends_with("\n\n"));
}

#[test]
fn github_output_format() {
    let problems = lint("key: value   \n");
    let reports = [FileReport {
        path: "test.yaml",
        problems: &problems,
    }];
    let out = render(&reports, OutputFormat::Github, false);
    assert!(out.starts_with("::group::test.yaml\n"), "{out:?}");
    assert!(
        out.contains(
            "::warning file=test.yaml,line=1,col=1::1:1 \
             [document-start] missing document start \"---\"\n"
        ),
        "{out:?}"
    );
    assert!(out.ends_with("::endgroup::\n\n"), "{out:?}");
}

#[test]
fn format_aliases() {
    assert_eq!(OutputFormat::parse("parsable"), Some(OutputFormat::Text));
    assert_eq!(
        OutputFormat::parse("standard"),
        Some(OutputFormat::Yamllint)
    );
    assert_eq!(OutputFormat::parse("auto"), Some(OutputFormat::Auto));
    assert_ne!(OutputFormat::Auto.resolve(), OutputFormat::Auto);
}

#[test]
fn no_warnings_filter() {
    let problems = lint("key: value   \n"); // warning (doc-start) + error (trailing)
    let reports = [FileReport {
        path: "t.yaml",
        problems: &problems,
    }];
    let out = render(&reports, OutputFormat::Text, true);
    assert!(out.contains("trailing spaces"));
    assert!(!out.contains("document start"));
}
