// fastymllint — a fast, drop-in replacement for yamllint written in Rust.
// Based on the work of yamllint (Copyright (C) 2016 Adrien Vergé and
// contributors, GPL-3.0-or-later).
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]

//! fastymllint CLI — a drop-in replacement for yamllint, with auto-fix and
//! auto-format modes.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use fastymllint::config::{YamlLintConfig, find_project_config_filepath};
use fastymllint::fix::{fix_content, unified_diff};
use fastymllint::linter::Level;
use fastymllint::output::{FileReport, OutputFormat, render};
use fastymllint::runner::{FileResult, find_files_recursively, lint_files};

/// Exit codes. Linting follows yamllint: 0 OK, 1 errors, 2 warnings in
/// strict mode. Fix/format dry runs exit with EXIT_DRY_RUN_CHANGES when
/// changes would be made; real failures exit with EXIT_SOFTWARE_FAILURE.
const EXIT_PROBLEMS: u8 = 1;
const EXIT_STRICT_WARNINGS: u8 = 2;
const EXIT_DRY_RUN_CHANGES: u8 = 3;
const EXIT_SOFTWARE_FAILURE: u8 = 255;

#[derive(Parser)]
#[command(
    name = "fastymllint",
    version,
    about = "A fast, drop-in replacement for yamllint, with auto-fix and auto-format",
    args_conflicts_with_subcommands = false
)]
struct Cli {
    /// Files or directories to lint ('-' for standard input)
    files: Vec<String>,

    /// Path to a custom configuration file
    #[arg(
        short = 'c',
        long = "config-file",
        global = true,
        conflicts_with = "config_data"
    )]
    config_file: Option<PathBuf>,

    /// Custom configuration (as YAML source)
    #[arg(short = 'd', long = "config-data", global = true)]
    config_data: Option<String>,

    /// Output format ('text' and 'json' are fastymllint extensions; the
    /// rest match yamllint, with 'auto' picking github/colored/standard
    /// from the environment)
    #[arg(short = 'f', long = "format", global = true, default_value = "auto",
          value_parser = ["parsable", "standard", "colored", "github", "auto",
                          "text", "json", "yamllint"])]
    format: String,

    /// Return non-zero exit code on warnings as well as errors
    #[arg(short = 's', long = "strict")]
    strict: bool,

    /// Output only error level problems
    #[arg(long = "no-warnings")]
    no_warnings: bool,

    /// List files to lint and exit
    #[arg(long = "list-files")]
    list_files: bool,

    /// Number of parallel jobs (default: number of CPUs, 1 disables
    /// parallelism)
    #[arg(short = 'j', long = "jobs", global = true)]
    jobs: Option<usize>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Automatically fix problems (whitespace, markers, spacing, ...)
    Fix {
        /// Files or directories to fix
        files: Vec<String>,

        /// Also apply fixes that may change how the document is parsed
        /// (truthy normalization, quoting, re-indentation)
        #[arg(long = "unsafe")]
        unsafe_fixes: bool,

        /// Show the diff of what would change without writing files
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
    /// Reformat files according to the yamllint style rules (safe fixes
    /// only)
    Format {
        /// Files or directories to format
        files: Vec<String>,

        /// Show the diff of what would change without writing files
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
}

fn load_config(cli: &Cli) -> Result<YamlLintConfig, String> {
    if let Some(data) = &cli.config_data {
        let data = if !data.is_empty() && !data.contains(':') {
            format!("extends: {data}")
        } else {
            data.clone()
        };
        return YamlLintConfig::from_content(&data).map_err(|e| e.to_string());
    }
    if let Some(file) = &cli.config_file {
        return YamlLintConfig::from_file(file).map_err(|e| e.to_string());
    }
    if let Ok(env_config) = std::env::var("YAMLLINT_CONFIG_FILE") {
        let path = if let Some(rest) = env_config.strip_prefix("~/") {
            match std::env::var("HOME") {
                Ok(home) => PathBuf::from(home).join(rest),
                Err(_) => PathBuf::from(env_config),
            }
        } else {
            PathBuf::from(env_config)
        };
        return YamlLintConfig::from_file(&path).map_err(|e| e.to_string());
    }
    if let Some(project_config) = find_project_config_filepath(Path::new(".")) {
        return YamlLintConfig::from_file(&project_config).map_err(|e| e.to_string());
    }
    // User-global config.
    let user_global = match std::env::var("XDG_CONFIG_HOME") {
        Ok(xdg) => PathBuf::from(xdg).join("yamllint").join("config"),
        Err(_) => match std::env::var("HOME") {
            Ok(home) => PathBuf::from(home)
                .join(".config")
                .join("yamllint")
                .join("config"),
            Err(_) => PathBuf::new(),
        },
    };
    if user_global.is_file() {
        return YamlLintConfig::from_file(&user_global).map_err(|e| e.to_string());
    }
    YamlLintConfig::from_content("extends: default").map_err(|e| e.to_string())
}

fn split_stdin(files: &[String]) -> (Vec<PathBuf>, bool) {
    let mut paths = Vec::new();
    let mut stdin = false;
    for f in files {
        if f == "-" {
            stdin = true;
        } else {
            paths.push(PathBuf::from(f));
        }
    }
    (paths, stdin)
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let conf = match load_config(&cli) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(EXIT_SOFTWARE_FAILURE);
        }
    };

    match &cli.command {
        None => lint_command(&cli, &conf),
        Some(Command::Fix {
            files,
            unsafe_fixes,
            dry_run,
        }) => fix_command(&cli, &conf, files, *unsafe_fixes, *dry_run, "fixed"),
        Some(Command::Format { files, dry_run }) => {
            fix_command(&cli, &conf, files, false, *dry_run, "reformatted")
        }
    }
}

fn lint_command(cli: &Cli, conf: &YamlLintConfig) -> ExitCode {
    let (paths, use_stdin) = split_stdin(&cli.files);

    // Like yamllint, missing input is a usage error (exit 2) even with
    // --list-files.
    if paths.is_empty() && !use_stdin {
        eprintln!("error: at least one file or directory (or '-' for stdin) is required");
        return ExitCode::from(2);
    }

    let files = find_files_recursively(&paths, conf);

    if cli.list_files {
        for file in &files {
            let display = file.to_string_lossy();
            let display = display.strip_prefix("./").unwrap_or(&display);
            if !conf.is_file_ignored(display) {
                println!("{}", file.display());
            }
        }
        return ExitCode::SUCCESS;
    }

    let format = OutputFormat::parse(&cli.format)
        .expect("validated by clap")
        .resolve();

    let mut results: Vec<FileResult> = lint_files(&files, conf, cli.jobs);

    if use_stdin {
        let mut data = Vec::new();
        if let Err(e) = std::io::stdin().read_to_end(&mut data) {
            eprintln!("{e}");
            return ExitCode::from(EXIT_SOFTWARE_FAILURE);
        }
        match fastymllint::decoder::auto_decode(&data) {
            Ok(content) => {
                let problems = fastymllint::linter::run(&content, conf, Some(""));
                results.push(FileResult {
                    path: PathBuf::from("stdin"),
                    display_path: "stdin".to_string(),
                    problems,
                    error: None,
                });
            }
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(EXIT_SOFTWARE_FAILURE);
            }
        }
    }

    // I/O errors abort the run.
    for result in &results {
        if let Some(error) = &result.error {
            eprintln!("{error}");
            return ExitCode::from(EXIT_SOFTWARE_FAILURE);
        }
    }

    let reports: Vec<FileReport> = results
        .iter()
        .map(|r| FileReport {
            // Show the path as given on the command line.
            path: if r.display_path == "stdin" {
                "stdin"
            } else {
                r.path.to_str().unwrap_or(&r.display_path)
            },
            problems: &r.problems,
        })
        .collect();

    let out = render(&reports, format, cli.no_warnings);
    print!("{out}");

    let max_level = results
        .iter()
        .flat_map(|r| r.problems.iter())
        .map(|p| p.level)
        .max();

    match max_level {
        Some(Level::Error) => ExitCode::from(EXIT_PROBLEMS),
        Some(Level::Warning) if cli.strict => ExitCode::from(EXIT_STRICT_WARNINGS),
        _ => ExitCode::SUCCESS,
    }
}

fn fix_command(
    _cli: &Cli,
    conf: &YamlLintConfig,
    files: &[String],
    unsafe_fixes: bool,
    dry_run: bool,
    verb: &str,
) -> ExitCode {
    let (paths, use_stdin) = split_stdin(files);
    if use_stdin {
        eprintln!("error: stdin is not supported in fix/format mode");
        return ExitCode::from(2);
    }
    if paths.is_empty() {
        eprintln!("error: at least one file or directory is required");
        return ExitCode::from(2);
    }

    let files = find_files_recursively(&paths, conf);

    let mut changed_count = 0usize;
    for file in &files {
        let display = file.to_string_lossy().to_string();
        let display_path = display.strip_prefix("./").unwrap_or(&display);
        if conf.is_file_ignored(display_path) {
            continue;
        }

        let data = match std::fs::read(file) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", fastymllint::runner::os_error_message(&e, file));
                return ExitCode::from(EXIT_SOFTWARE_FAILURE);
            }
        };
        let content = match fastymllint::decoder::auto_decode(&data) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: {e}", file.display());
                return ExitCode::from(EXIT_SOFTWARE_FAILURE);
            }
        };

        let result = fix_content(&content, conf, unsafe_fixes);
        if result.changed {
            changed_count += 1;
            if dry_run {
                print!("{}", unified_diff(display_path, &content, &result.fixed));
            } else {
                if let Err(e) = std::fs::write(file, &result.fixed) {
                    eprintln!("{}: {e}", file.display());
                    return ExitCode::from(EXIT_SOFTWARE_FAILURE);
                }
                println!("{display_path}: {verb}");
            }
        }
    }

    if dry_run && changed_count > 0 {
        eprintln!(
            "{changed_count} file{} would be {verb}",
            if changed_count == 1 { "" } else { "s" }
        );
        return ExitCode::from(EXIT_DRY_RUN_CHANGES);
    }
    if !dry_run {
        if changed_count > 0 {
            eprintln!(
                "{changed_count} file{} {verb}",
                if changed_count == 1 { "" } else { "s" }
            );
        } else {
            eprintln!("nothing to do");
        }
    }
    ExitCode::SUCCESS
}
