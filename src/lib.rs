//! MegaLinter - A high-performance, multi-language linter
//!
//! This library provides a framework for linting multiple programming languages
//! with support for parallel file processing, auto-fixing, and formatting.

pub mod config;
pub mod diagnostic;
pub mod language;
pub mod languages;
pub mod rule;
pub mod runner;

pub use config::Config;
pub use diagnostic::{Diagnostic, Severity};
pub use language::Language;
pub use rule::Rule;
pub use runner::LintRunner;
