# MegaLinter Architecture

MegaLinter is designed to be a high-performance, extensible, multi-language linter. This document outlines the core architectural components and their interactions.

## Core Concepts

### 1. The `Language` Trait
The `Language` trait is the primary extension point for adding support for new file types. Each language implementation (e.g., YAML, JSON) must provide:
- Name and file extensions.
- Logic for detection.
- `lint`, `fix`, and `format` methods.

### 2. The `Rule` Trait
Within a language, individual checks are implemented as `Rule`s. A `Rule`:
- Has a unique name and description.
- Takes a `RuleContext` and returns `Diagnostic`s.
- Can optionally indicate if it's auto-fixable.

### 3. `RuleContext`
Provides rules with the necessary information to perform checks:
- Raw file content.
- Pre-split lines (for line-based rules).
- File path.

### 4. `Diagnostic`
Represents a found issue:
- Rule name.
- Message.
- Severity (Error, Warning, Hint).
- Location (Line, Column).
- `Fix` information.

### 5. `LintRunner`
The engine that orchestrates the linting process:
- Discovers files to lint using `WalkDir`.
- Detects the appropriate `Language` for each file.
- Executes linting in parallel using `rayon`.
- Collects and returns `FileResult`s.

## Project Structure

- `src/main.rs`: CLI entry point, handles command-line arguments and output formatting.
- `src/lib.rs`: Library entry point, exports core traits and types.
- `src/language.rs`: Defines the `Language` trait.
- `src/rule.rs`: Defines the `Rule` trait and `RuleContext`.
- `src/diagnostic.rs`: Defines `Diagnostic`, `Severity`, and `Location`.
- `src/runner.rs`: Implements the parallel `LintRunner`.
- `src/languages/`: Contains language-specific implementations.
  - `src/languages/mod.rs`: Language registry.
  - `src/languages/yaml/`: YAML-specific logic and rules.

## Data Flow

1. **Input**: User provides paths via CLI.
2. **Discovery**: `LintRunner` walks the paths and identifies files supported by registered `Language`s.
3. **Processing**: For each file, `LintRunner` (in parallel):
    a. Reads file content.
    b. Calls `Language::lint`.
    c. `Language::lint` iterates through its `Rule`s and calls `Rule::check`.
    d. `Rule`s return `Diagnostic`s.
4. **Aggregation**: `LintRunner` collects all `FileResult`s.
5. **Output**: `main.rs` formats and displays the results to the user.

## Testing Strategy

### Unit Tests
- **Rule Tests**: Each rule should be tested in isolation. Tests should cover both positive cases (valid content) and negative cases (content that should trigger a diagnostic).
- **Location Verification**: Ensure that diagnostics report the correct line and column numbers.
- **Fix Verification**: For fixable rules, verify that the `fix` method (or helper function) produces the expected output.

Example rule test (in `tests/yaml_tests.rs`):
```rust
#[test]
fn test_trailing_spaces_detection() {
    let content = "key: value   \nother: ok\n";
    let ctx = RuleContext::new(content);
    let diags = TrailingSpaces.check(&ctx);

    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].location.line, 1);
    assert!(diags[0].message.contains("trailing"));
}
```

### Integration Tests
- **Language Integration**: Test that the `Language` implementation correctly identifies and lints files with the appropriate extensions.
- **Runner Integration**: Use `tempfile` and `WalkDir` to test the `LintRunner`'s ability to discover and process multiple files in parallel.
- **CLI Integration**: (Planned) Test the end-to-end flow from the command line interface.

### Performance Tests
- Use large YAML/JSON files to ensure that the linter remains fast and efficient under load.
- Measure the impact of parallelism with different thread counts (via `RAYON_NUM_THREADS`).

## Configuration System

MegaLinter aims to support multiple configuration formats:
1.  **Global Config**: `.megalinter.toml` for project-wide settings.
2.  **Language-Specific Config**: Support for existing standards like `.yamllint` or `package.json` for JS-related linters.
3.  **Inline Config**: Disabling rules via comments within files.
- **Parallelism**: Uses `rayon` for multi-threaded file processing.
- **Minimal Allocations**: Rules should aim to work with string slices (`&str`) where possible.
- **Lazy Loading**: Languages and rules are registered at startup but only invoked as needed.
