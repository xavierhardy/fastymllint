//! Integration tests for YAML linting

use tempfile::TempDir;
use std::fs;

use megalinter::{Config, LintRunner, Diagnostic};
use megalinter::languages::yaml::YamlLanguage;
use megalinter::Language;

mod yaml_rules {
    use super::*;
    use megalinter::rule::{Rule, RuleContext};
    use megalinter::languages::yaml::rules::*;

    #[test]
    fn test_trailing_spaces_detection() {
        let content = "key: value   \nother: ok\n";
        let ctx = RuleContext::new(content);
        let diags = TrailingSpaces.check(&ctx);
        
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].location.line, 1);
        assert!(diags[0].message.contains("trailing"));
    }

    #[test]
    fn test_trailing_spaces_fix() {
        use megalinter::languages::yaml::rules::fix_trailing_spaces;
        
        let content = "hello: world   \nfoo: bar  \n";
        let fixed = fix_trailing_spaces(content);
        assert_eq!(fixed, "hello: world\nfoo: bar\n");
    }

    #[test]
    fn test_line_length_default() {
        let ctx = RuleContext::new("short: line\n");
        let rule = LineLength::default();
        let diags = rule.check(&ctx);
        
        assert!(diags.is_empty());
    }

    #[test]
    fn test_line_length_exceeded() {
        let long_line = "word ".repeat(20); // 100 chars
        let content = format!("key: {}\n", long_line);
        let ctx = RuleContext::new(&content);
        let rule = LineLength::default(); // 80 chars
        let diags = rule.check(&ctx);
        
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("too long"));
    }

    #[test]
    fn test_newline_at_end_missing() {
        let content = "key: value";
        let ctx = RuleContext::new(content);
        let diags = NewLineAtEndOfFile.check(&ctx);
        
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn test_newline_at_end_present() {
        let content = "key: value\n";
        let ctx = RuleContext::new(content);
        let diags = NewLineAtEndOfFile.check(&ctx);
        
        assert!(diags.is_empty());
    }

    #[test]
    fn test_document_start_missing() {
        let content = "key: value\n";
        let ctx = RuleContext::new(content);
        let diags = DocumentStart.check(&ctx);
        
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("---"));
    }

    #[test]
    fn test_document_start_present() {
        let content = "---\nkey: value\n";
        let ctx = RuleContext::new(content);
        let diags = DocumentStart.check(&ctx);
        
        assert!(diags.is_empty());
    }

    #[test]
    fn test_key_duplicates_detected() {
        let content = "name: first\nvalue: 1\nname: second\n";
        let ctx = RuleContext::new(content);
        let diags = KeyDuplicates.check(&ctx);
        
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("duplication"));
        assert!(diags[0].message.contains("name"));
    }

    #[test]
    fn test_key_duplicates_nested_ok() {
        let content = "parent:\n  name: child\nname: root\n";
        let ctx = RuleContext::new(content);
        let diags = KeyDuplicates.check(&ctx);
        
        assert!(diags.is_empty(), "Nested keys at different levels should not conflict");
    }

    #[test]
    fn test_empty_lines_max() {
        let content = "key: value\n\n\n\n\nother: value\n";
        let ctx = RuleContext::new(content);
        let rule = EmptyLines::new(2);
        let diags = rule.check(&ctx);
        
        assert!(!diags.is_empty());
        assert!(diags[0].message.contains("blank lines"));
    }

    #[test]
    fn test_indentation_consistent() {
        let content = "root:\n  child:\n    grandchild: value\n";
        let ctx = RuleContext::new(content);
        let rule = Indentation::new(2);
        let diags = rule.check(&ctx);
        
        assert!(diags.is_empty());
    }

    #[test]
    fn test_indentation_inconsistent() {
        let content = "root:\n  child:\n   bad: value\n";  // 3 spaces instead of 4
        let ctx = RuleContext::new(content);
        let rule = Indentation::new(2);
        let diags = rule.check(&ctx);
        
        assert!(!diags.is_empty());
    }
}

mod yaml_language {
    use super::*;

    #[test]
    fn test_yaml_detection() {
        let lang = YamlLanguage::new();
        
        assert!(lang.detect(std::path::Path::new("test.yaml")));
        assert!(lang.detect(std::path::Path::new("config.yml")));
        assert!(lang.detect(std::path::Path::new(".yamllint")));
        assert!(!lang.detect(std::path::Path::new("file.json")));
        assert!(!lang.detect(std::path::Path::new("script.py")));
    }

    #[test]
    fn test_yaml_lint() {
        let lang = YamlLanguage::new();
        let config = Config::default();
        
        let content = "key: value   \n"; // trailing space
        let diags = lang.lint(content, &config);
        
        assert!(!diags.is_empty());
    }

    #[test]
    fn test_yaml_fix() {
        let lang = YamlLanguage::new();
        let config = Config::default();
        
        let content = "key: value   \nother: test  ";
        let fixed = lang.fix(content, &config).unwrap();
        
        assert!(!fixed.contains("   "), "Trailing spaces should be removed");
        assert!(fixed.ends_with('\n'), "Should end with newline");
    }
}

mod integration {
    use super::*;

    #[test]
    fn test_lint_runner() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.yaml");
        fs::write(&file, "key: value   \n").unwrap();
        
        let config = Config::new(temp.path());
        let runner = LintRunner::new(config);
        let results = runner.lint_all();
        
        assert_eq!(results.len(), 1);
        assert!(!results[0].diagnostics.is_empty());
    }

    #[test]
    fn test_fix_runner() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.yaml");
        fs::write(&file, "key: value   ").unwrap(); // trailing space, no newline
        
        let config = Config::new(temp.path());
        let runner = LintRunner::new(config);
        let fixed = runner.fix_all().unwrap();
        
        assert_eq!(fixed.len(), 1);
        
        let content = fs::read_to_string(&file).unwrap();
        assert!(!content.contains("   "));
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_parallel_processing() {
        let temp = TempDir::new().unwrap();
        
        // Create multiple files
        for i in 0..10 {
            let file = temp.path().join(format!("file{}.yaml", i));
            fs::write(&file, format!("key{}: value\n", i)).unwrap();
        }
        
        let config = Config::new(temp.path());
        let runner = LintRunner::new(config);
        let results = runner.lint_all();
        
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_valid_yaml_no_errors() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("valid.yaml");
        let content = r#"---
name: example
version: 1.0.0
metadata:
  author: test
  tags:
    - yaml
    - linter
"#;
        fs::write(&file, content).unwrap();
        
        let config = Config::new(temp.path());
        let runner = LintRunner::new(config);
        let results = runner.lint_all();
        
        // With default rules (document-start disabled), this should pass most checks
        // Only document-start/end would fail if enabled
        let errors: Vec<_> = results.iter()
            .flat_map(|r| r.diagnostics.iter())
            .filter(|d| d.rule != "document-start" && d.rule != "document-end")
            .collect();
        
        assert!(errors.is_empty(), "Valid YAML should have no errors (except optional doc markers): {:?}", errors);
    }
}

mod config {
    use super::*;
    use megalinter::languages::yaml::config::YamllintConfig;

    #[test]
    fn test_yamllint_config_parse() {
        let yaml = r#"
extends: default
rules:
  line-length:
    max: 100
  trailing-spaces: enable
"#;
        let config: YamllintConfig = serde_yaml_ng::from_str(yaml).unwrap();
        
        assert_eq!(config.extends, Some("default".to_string()));
        assert!(config.rules.contains_key("line-length"));
        assert!(config.rules.contains_key("trailing-spaces"));
    }

    #[test]
    fn test_yamllint_default_config() {
        let config = YamllintConfig::default_config();
        
        assert!(config.is_rule_enabled("trailing-spaces"));
        assert!(config.is_rule_enabled("line-length"));
        assert!(!config.is_rule_enabled("document-start"));
    }

    #[test]
    fn test_yamllint_relaxed_config() {
        let config = YamllintConfig::relaxed_config();
        
        // Relaxed config should still have some rules
        assert!(config.is_rule_enabled("trailing-spaces"));
        assert!(!config.is_rule_enabled("truthy")); // Disabled in relaxed
    }
}
