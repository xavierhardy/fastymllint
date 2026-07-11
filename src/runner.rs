//! File discovery and parallel linting.

use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::config::YamlLintConfig;
use crate::decoder::auto_decode;
use crate::linter::{self, LintProblem};

/// Find files to lint:
/// directories are walked recursively (only files matching `yaml-files` and
/// not ignored), explicit files are always returned.
pub fn find_files_recursively(items: &[PathBuf], conf: &YamlLintConfig) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for item in items {
        if item.is_dir() {
            let walker = walkdir::WalkDir::new(item)
                .sort_by_file_name()
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());
            for entry in walker {
                let path = entry.path().to_path_buf();
                let path_str = path.to_string_lossy();
                if conf.is_yaml_file(&path_str) && !conf.is_file_ignored(&path_str) {
                    files.push(path);
                }
            }
        } else {
            files.push(item.clone());
        }
    }
    files
}

pub struct FileResult {
    pub path: PathBuf,
    /// Path as displayed/matched (leading `./` stripped).
    pub display_path: String,
    pub problems: Vec<LintProblem>,
    /// I/O or decoding error, if the file could not be processed.
    pub error: Option<String>,
}

/// Format an I/O error the way Python renders `OSError` (what yamllint
/// prints): `[Errno N] <strerror>: '<path>'`. Rust's own `Display` appends
/// ` (os error N)` instead, so strip that and prepend the errno.
pub fn os_error_message(e: &std::io::Error, path: &Path) -> String {
    match e.raw_os_error() {
        Some(errno) => {
            let msg = e.to_string();
            let msg = msg
                .strip_suffix(&format!(" (os error {errno})"))
                .unwrap_or(&msg)
                .to_string();
            format!("[Errno {errno}] {msg}: '{}'", path.display())
        }
        None => format!("{e}: '{}'", path.display()),
    }
}

pub fn lint_one(path: &Path, conf: &YamlLintConfig) -> FileResult {
    let display_path = {
        let s = path.to_string_lossy().to_string();
        s.strip_prefix("./").map(str::to_string).unwrap_or(s)
    };

    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            return FileResult {
                path: path.to_path_buf(),
                display_path,
                problems: Vec::new(),
                error: Some(os_error_message(&e, path)),
            };
        }
    };
    let content = match auto_decode(&data) {
        Ok(c) => c,
        Err(e) => {
            return FileResult {
                path: path.to_path_buf(),
                display_path,
                problems: Vec::new(),
                error: Some(format!("{}: {e}", path.display())),
            };
        }
    };

    let problems = linter::run(&content, conf, Some(&display_path));
    FileResult {
        path: path.to_path_buf(),
        display_path,
        problems,
        error: None,
    }
}

/// Lint files in parallel, preserving input order in the result.
pub fn lint_files(
    files: &[PathBuf],
    conf: &YamlLintConfig,
    jobs: Option<usize>,
) -> Vec<FileResult> {
    let run = || {
        files
            .par_iter()
            .map(|path| lint_one(path, conf))
            .collect::<Vec<_>>()
    };

    match jobs {
        // No point spinning up a thread pool for a single file.
        _ if files.len() <= 1 => files.iter().map(|path| lint_one(path, conf)).collect(),
        Some(1) => files.iter().map(|path| lint_one(path, conf)).collect(),
        Some(n) => rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .map(|pool| pool.install(run))
            .unwrap_or_else(|_| run()),
        None => run(),
    }
}
