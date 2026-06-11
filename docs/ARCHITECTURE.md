# fastymllint Architecture

fastymllint is a drop-in replacement for yamllint. Output parity is the
design constraint that shapes everything: yamllint's rules depend on the
exact token marks (character pointer, line, column) of PyYAML's scanner, so
fastymllint is built on a faithful Rust port of that scanner rather than an
off-the-shelf YAML parser.

## Layers

```
src/
├── pyyaml/            Port of the PyYAML pieces yamllint relies on (MIT)
│   ├── scanner.rs     Token scanner — exact marks, values, styles, errors
│   ├── parser.rs      Event parser state machine — syntax errors + events
│   ├── loader.rs      Safe YAML document loader (used for config files)
│   ├── value.rs       Plain YAML value tree
│   ├── resolver.rs    Implicit tag resolution (used by quoted-strings)
│   ├── tokens.rs      Token types and PyYAML token ids
│   ├── error.rs       Mark + error types (problem & mark drive messages)
│   └── pyrepr.rs      Python repr() formatting for error message parity
├── linter.rs          Lint pipeline: token/comment/line generators,
│                      inline `# yamllint disable...` directives, syntax
│                      error merging
├── rules/             All 23 yamllint rules, one module each, plus
│                      option parsing/validation and dispatch (mod.rs)
├── config.rs          Configuration: extends (default/relaxed/file),
│                      levels, ignore / yaml-files patterns (gitignore
│                      semantics via the `ignore` crate)
├── decoder.rs         BOM-based encoding detection (UTF-8/16/32)
├── runner.rs          File discovery + parallel linting (rayon)
├── output.rs          text / json / yamllint output formats
├── fix.rs             Auto-fix & format engine (safe vs unsafe fixes)
└── main.rs            CLI (lint by default; `fix` / `format` subcommands)
```

## The scanner port

`src/pyyaml/scanner.rs` implements the same simple-key bookkeeping,
indentation stack, token queue and mark semantics as PyYAML's scanner. The
buffer is a `Vec<char>` with a `'\0'` sentinel, exactly like PyYAML's
reader, so `Mark.pointer` is a character index that matches
PyYAML's. This is verified by a differential test: `src/bin/dump_tokens.rs`
and `tests/dump_tokens.py` print canonical token dumps that must be
identical across the whole test corpus.

`src/pyyaml/parser.rs` implements the event parser state machine. It
detects the first syntax error with PyYAML's exact message and position
(reported as `syntax error: ... (syntax)`), and feeds `loader.rs`, which
composes events into plain values for configuration files. The whole crate
is 100% safe Rust (`#![forbid(unsafe_code)]`); no C-derived YAML library
is used.

## The linting pipeline

`linter::run` works as follows:

1. `# yamllint disable-file` on the first line skips the file.
2. The parser detects a possible syntax error.
3. Tokens (with prev/next/nextnext context), comments (found in the gaps
   between tokens) and physical lines are generated and merged in line
   order; each element is dispatched to the enabled rules of the matching
   type (token / comment / line), in configuration order.
4. Problems are cached per line and filtered through the inline-directive
   state (`disable`, `enable`, `disable-line`).
5. The syntax error, if any, is merged at the right position, suppressing a
   cosmetic problem at the same location.

Rules live in `src/rules/`, one module per rule, each producing identical
message strings, positions and ordering to the upstream rule. Stateful
rules (anchors, indentation, key-duplicates, key-ordering, quoted-strings,
truthy) keep their per-file context in a `RuleRunner`.

## Fixing and formatting

`fix.rs` repeatedly lints and applies textual edits derived from reported
problems until a fixed point (bounded number of passes). Fixers are
classified:

- **safe** — cannot change parsed semantics: trailing spaces, final
  newline, newline type, blank-line trimming, document start/end markers,
  comment spacing/indentation, spacing around `:` `,` `-` `{}` `[]`;
- **unsafe** (`fix --unsafe`) — may change semantics for some consumers:
  truthy normalization, quoting octal values, adding/removing string
  quotes, line re-indentation.

`format` applies the safe set only. `--dry-run` renders a unified diff and
uses exit code 3 when changes would be required (distinct from 255 for
software failures).

## Testing strategy

- `tests/parity.rs` / `tests/parity.sh`: run the real yamllint (from
  `.venv`) and fastymllint side by side over `examples/yaml/` and
  `tests/data/tricky/` with several configurations; stdout (yamllint
  format) and exit codes must match exactly.
- Token-level differential tests against PyYAML (see above).
- `tests/lint_test.rs`: pure-Rust integration tests for linting, config,
  fixing and output formats.
- `bench/bench.sh`: speed comparison against yamllint (single small file,
  single large file, many files).
