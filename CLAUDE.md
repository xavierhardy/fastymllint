# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

fastymllint is a drop-in replacement for [yamllint](https://github.com/adrienverge/yamllint) written in 100% safe Rust (`#![forbid(unsafe_code)]`), plus auto-fix (`fix`) and auto-format (`format`) subcommands. **Byte-for-byte output parity with yamllint is the design constraint that shapes everything** — messages, positions, severities, ordering, and exit codes must match yamllint exactly (in `-f yamllint` format). License is GPL-3.0-or-later.

## Commands

```bash
./build.sh                        # cargo build --release
./test.sh                         # cargo test (unit + integration + parity)
./lint-format.sh                  # cargo fmt --all && cargo clippy -- -D warnings
./run.sh <args>                   # cargo run --release -- <args>

# One-time setup: the reference yamllint, required by parity tests and benchmarks
python3 -m venv .venv && .venv/bin/pip install yamllint

# Single test
cargo test --test lint_test <test_name>   # integration tests in tests/lint_test.rs
cargo test --test parity                  # side-by-side parity vs yamllint
cargo test <name>                         # unit tests (in src/ modules)

./tests/parity.sh                 # quick parity sweep over the example corpus
./bench/bench.sh                  # benchmarks vs yamllint
```

Verification for every task: `./lint-format.sh`, `./test.sh`, and `./build.sh` must all pass. clippy warnings are errors.

## Task tracking

- Consult `TASKS.md` before starting work; mark tasks done as you complete them.
- Record completed work in `docs/DONE_TASKS.md` as `- YYYY-MM-DD: Task description`.
- Update relevant docs (README, `docs/ARCHITECTURE.md`, `docs/rules/yaml/*.md`) for every change that affects behavior.

## Architecture

Full details in `docs/ARCHITECTURE.md`. The big picture:

yamllint's rules depend on the exact token marks (character pointer, line, column) of PyYAML's scanner, so fastymllint is built on a faithful Rust port of that scanner (`src/pyyaml/`, MIT-derived) rather than an off-the-shelf YAML parser. The buffer is a `Vec<char>` with a `'\0'` sentinel exactly like PyYAML's reader, so `Mark.pointer` is a character index matching PyYAML's. `src/pyyaml/pyrepr.rs` reproduces Python `repr()` formatting so error messages match character for character.

Data flow:

1. `src/main.rs` — CLI (lint by default; `fix`/`format` subcommands).
2. `src/runner.rs` — file discovery + parallel linting (rayon, `-j/--jobs`).
3. `src/linter.rs` — the lint pipeline: generates tokens (with prev/next/nextnext context), comments (found in gaps between tokens), and physical lines; merges them in line order; dispatches each to enabled rules by type (token/comment/line) in configuration order; filters through inline `# yamllint disable...` directives; merges the first syntax error at the right position.
4. `src/rules/` — all 23 yamllint rules, one module each; option parsing/validation and dispatch in `mod.rs`. Stateful rules (anchors, indentation, key-duplicates, key-ordering, quoted-strings, truthy) keep per-file context in a `RuleRunner`.
5. `src/output.rs` — `text` (default), `json`, and `yamllint` (byte-identical to yamllint's standard output) formats.

Other key pieces:

- `src/config.rs` — yamllint-compatible config: `extends` (default/relaxed/file — bundled defaults in `src/conf/`), per-rule levels, `ignore`/`ignore-from-file` (gitignore semantics via the `ignore` crate), `yaml-files` patterns.
- `src/fix.rs` — fix/format engine: repeatedly lints and applies textual edits until a fixed point (bounded passes). Fixers are classified **safe** (whitespace/marker-only, cannot change parsed semantics; `format` applies only these) vs **unsafe** (`fix --unsafe`: truthy normalization, quoting octals, adding/removing quotes, re-indentation). `--dry-run` prints a unified diff and exits 3 when changes would be required.
- `src/decoder.rs` — BOM-based encoding detection (UTF-8/16/32).
- `src/bin/dump_tokens.rs` — canonical token dumps for the differential test against PyYAML (`tests/dump_tokens.py`).

Exit codes: 0 success; 1 lint errors; 2 warnings in `--strict` / usage errors; 3 dry-run changes needed; 255 software failure (bad config, I/O).

## Parity rules (critical)

- The behavioral reference is the yamllint source installed in `.venv` (`.venv/lib/python*/site-packages/yamllint/`). When in doubt about behavior, read that source, not intuition.
- Any change to the scanner, linter, or rules must keep `cargo test` and `./tests/parity.sh` green: identical stdout (yamllint format) and identical exit codes across the corpus (`examples/yaml/`, `tests/data/tricky/`) crossed with the configs in `tests/configs/` plus `default`/`relaxed`.
- When fixing a parity bug, add the offending YAML to `tests/data/tricky/` so it stays fixed.
- Scanner changes must also keep the token-level differential test green (`dump_tokens` vs `tests/dump_tokens.py`): identical token kinds, values, styles, and character-exact line/column/pointer marks.
- New safe fixers must come with semantic-preservation checks.
- Known intentional divergence: the `locale` config option is accepted but ignored (`key-ordering` always compares by code point).
