//! MegaLinter CLI - A high-performance, multi-language linter

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

use megalinter::{Config, LintRunner, Severity};

#[derive(Parser)]
#[command(name = "megalinter")]
#[command(author, version, about = "A high-performance, multi-language linter", long_about = None)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint files and report issues
    Lint {
        /// Files or directories to lint
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    
    /// Fix auto-fixable issues
    Fix {
        /// Files or directories to fix
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        
        /// Dry run - show what would be fixed without making changes
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Format files according to style rules
    Format {
        /// Files or directories to format
        #[arg(default_value = ".")]
        paths: Vec<PathBuf>,
        
        /// Check mode - exit with error if files need formatting
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    
    match run(cli) {
        Ok(has_errors) => {
            if has_errors {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "error".red().bold(), e);
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<bool> {
    match cli.command {
        Commands::Lint { paths, format } => {
            lint_command(&paths, &format)
        }
        Commands::Fix { paths, dry_run } => {
            fix_command(&paths, dry_run)
        }
        Commands::Format { paths, check } => {
            format_command(&paths, check)
        }
    }
}

fn lint_command(paths: &[PathBuf], _format: &str) -> Result<bool> {
    let mut has_errors = false;
    
    for path in paths {
        let config = Config::new(path);
        let runner = LintRunner::new(config);
        let results = runner.lint_all();
        
        for result in results {
            if let Some(error) = &result.error {
                eprintln!("{}: {}: {}", 
                    result.path.display().to_string().cyan(),
                    "error".red().bold(),
                    error
                );
                has_errors = true;
                continue;
            }
            
            if !result.diagnostics.is_empty() {
                has_errors = true;
                
                for diag in &result.diagnostics {
                    let severity_str = match diag.severity {
                        Severity::Error => "error".red().bold(),
                        Severity::Warning => "warning".yellow().bold(),
                        Severity::Hint => "hint".blue(),
                    };
                    
                    println!("{}:{}:{}: {} [{}] {}",
                        result.path.display().to_string().cyan(),
                        diag.location.line,
                        diag.location.column,
                        severity_str,
                        diag.rule.dimmed(),
                        diag.message
                    );
                }
            }
        }
    }
    
    if !has_errors {
        println!("{}", "All files passed linting!".green().bold());
    }
    
    Ok(has_errors)
}

fn fix_command(paths: &[PathBuf], dry_run: bool) -> Result<bool> {
    let mut fixed_count = 0;
    
    for path in paths {
        let config = Config::new(path);
        let runner = LintRunner::new(config);
        
        if dry_run {
            let results = runner.lint_all();
            for result in results {
                let fixable = result.diagnostics.iter().filter(|d| d.fix.is_some()).count();
                if fixable > 0 {
                    println!("{}: {} fixable issues",
                        result.path.display().to_string().cyan(),
                        fixable
                    );
                    fixed_count += fixable;
                }
            }
        } else {
            let fixed_files = runner.fix_all()?;
            for file in &fixed_files {
                println!("{}: {}", 
                    file.display().to_string().cyan(),
                    "fixed".green().bold()
                );
            }
            fixed_count += fixed_files.len();
        }
    }
    
    if fixed_count > 0 {
        if dry_run {
            println!("\n{} issues can be fixed (run without --dry-run to apply)", fixed_count);
        } else {
            println!("\n{} {} fixed", fixed_count, if fixed_count == 1 { "file" } else { "files" });
        }
    } else {
        println!("{}", "No issues to fix!".green().bold());
    }
    
    Ok(false)
}

fn format_command(paths: &[PathBuf], check: bool) -> Result<bool> {
    let mut needs_formatting = false;
    
    for path in paths {
        let config = Config::new(path);
        let runner = LintRunner::new(config);
        
        if check {
            let results = runner.lint_all();
            for result in results {
                if !result.diagnostics.is_empty() {
                    println!("{}: needs formatting",
                        result.path.display().to_string().cyan()
                    );
                    needs_formatting = true;
                }
            }
        } else {
            let fixed_files = runner.fix_all()?;
            for file in &fixed_files {
                println!("{}: formatted", file.display().to_string().cyan());
            }
        }
    }
    
    if check && needs_formatting {
        println!("\n{}", "Some files need formatting (run without --check to apply)".yellow());
    } else if !check {
        println!("{}", "All files formatted!".green().bold());
    }
    
    Ok(needs_formatting)
}
