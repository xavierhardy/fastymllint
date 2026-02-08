//! Parallel lint runner

use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::languages::LanguageRegistry;
use crate::{Config, Diagnostic};

/// Result of linting a single file
#[derive(Debug)]
pub struct FileResult {
    /// Path to the file
    pub path: PathBuf,
    /// Diagnostics found in the file
    pub diagnostics: Vec<Diagnostic>,
    /// Error if the file couldn't be processed
    pub error: Option<String>,
}

impl FileResult {
    pub fn success(path: PathBuf, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            path,
            diagnostics,
            error: None,
        }
    }

    pub fn error(path: PathBuf, error: impl Into<String>) -> Self {
        Self {
            path,
            diagnostics: Vec::new(),
            error: Some(error.into()),
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error.is_some() || !self.diagnostics.is_empty()
    }
}

/// The main lint runner
pub struct LintRunner {
    config: Config,
    registry: LanguageRegistry,
}

impl LintRunner {
    /// Create a new runner with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: LanguageRegistry::new(),
        }
    }

    /// Discover files to lint in the given directory
    pub fn discover_files(&self, root: &Path) -> Vec<PathBuf> {
        WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| self.registry.detect(e.path()).is_some())
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    /// Lint a single file
    pub fn lint_file(&self, path: &Path) -> FileResult {
        let lang = match self.registry.detect(path) {
            Some(l) => l,
            None => return FileResult::error(path.to_path_buf(), "Unknown file type"),
        };

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => return FileResult::error(path.to_path_buf(), e.to_string()),
        };

        let diagnostics = lang.lint(&content, &self.config);
        FileResult::success(path.to_path_buf(), diagnostics)
    }

    /// Lint multiple files in parallel
    pub fn lint_files(&self, files: &[PathBuf]) -> Vec<FileResult> {
        files.par_iter().map(|path| self.lint_file(path)).collect()
    }

    /// Lint all files in the root directory
    pub fn lint_all(&self) -> Vec<FileResult> {
        let files = self.discover_files(&self.config.root);
        self.lint_files(&files)
    }

    /// Fix a single file
    pub fn fix_file(&self, path: &Path) -> Result<bool> {
        let lang = self.registry.detect(path).context("Unknown file type")?;

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        let fixed = lang.fix(&content, &self.config)?;

        if fixed != content {
            fs::write(path, &fixed)
                .with_context(|| format!("Failed to write file: {}", path.display()))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Fix all files in the root directory
    pub fn fix_all(&self) -> Result<Vec<PathBuf>> {
        let files = self.discover_files(&self.config.root);
        let mut fixed_files = Vec::new();

        for path in files {
            if self.fix_file(&path)? {
                fixed_files.push(path);
            }
        }

        Ok(fixed_files)
    }
}
