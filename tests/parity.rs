//! Side-by-side parity tests: the real yamllint (from `.venv`) and
//! fastymllint are run on the same files and must produce identical output
//! (in yamllint's standard format) and identical exit codes.
//!
//! Setup: `python3 -m venv .venv && .venv/bin/pip install yamllint`

use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn yamllint() -> PathBuf {
    let path = repo_root().join(".venv/bin/yamllint");
    assert!(
        path.exists(),
        "yamllint not found at {path:?}; run: python3 -m venv .venv && .venv/bin/pip install yamllint"
    );
    path
}

fn corpus() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in ["examples/yaml", "tests/data/tricky"] {
        let dir = repo_root().join(dir);
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .unwrap_or_else(|_| panic!("missing corpus dir {dir:?}"))
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
            .collect();
        entries.sort();
        files.extend(entries);
    }
    assert!(!files.is_empty());
    files
}

struct Run {
    stdout: String,
    code: i32,
}

fn run(cmd: &Path, args: &[&str], files: &[PathBuf]) -> Run {
    let output = Command::new(cmd)
        .args(args)
        .args(files)
        .current_dir(repo_root())
        // Make `-f auto` deterministic (also on GitHub Actions runners).
        .env_remove("GITHUB_ACTIONS")
        .env_remove("GITHUB_WORKFLOW")
        .output()
        .unwrap_or_else(|e| panic!("failed to run {cmd:?}: {e}"));
    Run {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        code: output.status.code().unwrap_or(-1),
    }
}

fn compare(config_args: &[&str], files: &[PathBuf]) {
    let mut yl_args = vec!["-f", "standard"];
    yl_args.extend_from_slice(config_args);
    let yl = run(&yamllint(), &yl_args, files);

    let mut fy_args = vec!["-f", "yamllint"];
    fy_args.extend_from_slice(config_args);
    let fy = run(
        Path::new(env!("CARGO_BIN_EXE_fastymllint")),
        &fy_args,
        files,
    );

    assert_eq!(
        yl.stdout, fy.stdout,
        "output mismatch for {config_args:?} on {files:?}"
    );
    assert_eq!(
        yl.code, fy.code,
        "exit code mismatch for {config_args:?} on {files:?}"
    );
}

#[test]
fn parity_default_config_per_file() {
    for file in corpus() {
        compare(&["-d", "extends: default"], std::slice::from_ref(&file));
    }
}

#[test]
fn parity_relaxed_config_all_files() {
    let files = corpus();
    compare(&["-d", "extends: relaxed"], &files);
}

#[test]
fn parity_strict_all_rules_config() {
    let files = corpus();
    compare(&["-c", "tests/configs/strict_all.yaml"], &files);
}

#[test]
fn parity_quotes_required_config() {
    let files = corpus();
    compare(&["-c", "tests/configs/quotes_required.yaml"], &files);
}

/// Every output format yamllint knows must be byte-identical, including
/// `colored` (explicitly selecting it emits ANSI escapes even into a pipe)
/// and `auto` (which resolves to `standard` here: stdout is a pipe and the
/// GitHub env vars are removed by `run`).
#[test]
fn parity_output_formats() {
    let files = corpus();
    for format in ["parsable", "standard", "colored", "github", "auto"] {
        let yl = run(
            &yamllint(),
            &["-f", format, "-d", "extends: default"],
            &files,
        );
        let fy = run(
            Path::new(env!("CARGO_BIN_EXE_fastymllint")),
            &["-f", format, "-d", "extends: default"],
            &files,
        );
        assert_eq!(yl.stdout, fy.stdout, "output mismatch for -f {format}");
        assert_eq!(yl.code, fy.code, "exit code mismatch for -f {format}");
    }
}

#[test]
fn parity_no_warnings_and_strict() {
    let files = corpus();
    compare(&["-d", "extends: default", "--no-warnings"], &files);
    compare(&["-d", "extends: default", "-s"], &files);
}

#[test]
fn token_stream_matches_pyyaml() {
    let python = repo_root().join(".venv/bin/python");
    assert!(python.exists(), "missing .venv python");
    for file in corpus() {
        let ours = Command::new(env!("CARGO_BIN_EXE_dump_tokens"))
            .arg(&file)
            .output()
            .expect("run dump_tokens");
        let theirs = Command::new(&python)
            .arg(repo_root().join("tests/dump_tokens.py"))
            .arg(&file)
            .current_dir(repo_root())
            .output()
            .expect("run dump_tokens.py");
        assert_eq!(
            String::from_utf8_lossy(&ours.stdout),
            String::from_utf8_lossy(&theirs.stdout),
            "token stream mismatch for {file:?}"
        );
    }
}
