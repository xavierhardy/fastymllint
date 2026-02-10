# Contributing to MegaLinter

Thank you for your interest in contributing to MegaLinter!

## Getting Started

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/xhardy/megalinter
    cd megalinter
    ```
2.  **Run tests**:
    ```bash
    cargo test
    ```
3.  **Build the project**:
    ```bash
    cargo build
    ```

## Adding a New Language

1.  Create a new directory in `src/languages/`.
2.  Implement the `Language` trait.
3.  Register the new language in `src/languages/mod.rs`.
4.  Add unit tests for detection and basic linting.

## Adding a New Rule

1.  Create a new file in the appropriate language's `rules/` directory.
2.  Implement the `Rule` trait.
3.  Add the rule to the language's `RuleSet`.
4.  Add unit tests for the rule.

## Coding Standards

- Follow idiomatic Rust patterns.
- Ensure all tests pass before submitting a pull request.
- Add documentation for any new features or rules.
- If you are an AI assistant, please refer to [GEMINI.md](GEMINI.md).

## Workflow

1.  Check [TASKS.md](TASKS.md) for available tasks.
2.  Create a new branch for your work.
3.  Implement your changes and add tests.
4.  Update [DONE_TASKS.md](docs/DONE_TASKS.md) once finished.
5.  Submit a pull request.
