hello

# MegaLinter

A high-performance, multi-language linter written in Rust.

## Usage

MegaLinter provides `lint`, `fix`, and `format` commands to help maintain code quality.

### Installation

(Instructions on how to install MegaLinter will go here once available, e.g., `cargo install megalinter`)

### Lint

The `lint` command checks files for issues and reports them.

#### Syntax
```bash
megalinter lint [OPTIONS] [PATHS]...
```

#### Arguments
*   `[PATHS]...`: Files or directories to lint. Defaults to the current directory (`.`).

#### Options
*   `-c, --config <CONFIG>`: Path to a configuration file.
*   `-f, --format <FORMAT>`: Output format for the lint report (e.g., `text`, `json`). Defaults to `text`.

#### Examples
*   Lint all files in the current directory:
    ```bash
    megalinter lint
    ```
*   Lint specific files or directories:
    ```bash
    megalinter lint src tests/yaml_tests.rs
    ```
*   Lint with a custom configuration file and output in JSON format:
    ```bash
    megalinter lint --config .megalinter.toml --format json
    ```

### Fix

The `fix` command automatically fixes auto-fixable issues.

#### Syntax
```bash
megalinter fix [OPTIONS] [PATHS]...
```

#### Arguments
*   `[PATHS]...`: Files or directories to fix. Defaults to the current directory (`.`).

#### Options
*   `-c, --config <CONFIG>`: Path to a configuration file.
*   `--dry-run`: Show what would be fixed without making actual changes.

#### Examples
*   Fix auto-fixable issues in the current directory:
    ```bash
    megalinter fix
    ```
*   Perform a dry run to see what would be fixed:
    ```bash
    megalinter fix --dry-run src/my_file.yaml
    ```

### format

The `format` command formats files according to style rules.

#### Syntax
```bash
megalinter format [OPTIONS] [PATHS]...
```

#### Arguments
*   `[PATHS]...`: Files or directories to format. Defaults to the current directory (`.`).

#### Options
*   `-c, --config <CONFIG>`: Path to a configuration file.
*   `--check`: Exit with an error if files need formatting, without applying changes.

#### Examples
*   Format files in the current directory:
    ```bash
    megalinter format
    ```
*   Check if files need formatting:
    ```bash
    megalinter format --check src/
    ```

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [AI Instructions](GEMINI.md)
- [Tasks](TASKS.md)
- [Done Tasks](docs/DONE_TASKS.md)

## Development

### AI Assistance

This project uses specific guidelines for AI contributors. If you are using an AI assistant to help with development, please ensure it reads [GEMINI.md](GEMINI.md) first.
