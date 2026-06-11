# fastymllint

A fast, drop-in replacement for [yamllint](https://github.com/adrienverge/yamllint)
written in Rust — plus auto-fix and auto-format.

The linting pipeline is a faithful port of yamllint built on a Rust port of
PyYAML's token scanner, so diagnostics — messages, positions, severities and
ordering — match yamllint **byte for byte** (verified by side-by-side parity
tests). On top of that, fastymllint adds:

- **Speed**: typically 3–10× faster than yamllint, with parallel processing
  of multiple files (configurable with `-j/--jobs`).
- **Auto-fix** (`fastymllint fix`): automatically repairs problems. Two
  levels:
  - safe fixes (default): whitespace/marker-only edits that cannot change
    document semantics — trailing spaces, newlines, blank lines, spacing
    around `:`/`,`/`-`/braces/brackets, comment spacing and indentation,
    document start/end markers;
  - `--unsafe`: adds fixes that may change how some tools parse the document
    — truthy normalization (`yes` → `true`), quoting octal values,
    adding/removing string quotes, re-indenting lines.
- **Auto-format** (`fastymllint format`): normalizes style by applying all
  safe fixes.
- **Dry-run** (`--dry-run`): shows a unified diff without writing anything,
  and exits with a dedicated status code if changes would be made.
- **Output formats**: `text` (default, one grep-friendly line per problem),
  `json`, and `yamllint` (byte-identical to yamllint's standard output).

## Usage

```console
# Lint (yamllint-compatible CLI)
fastymllint file.yaml dir/ -          # files, directories, or stdin ('-')
fastymllint -c .yamllint file.yaml    # custom config file
fastymllint -d relaxed file.yaml      # inline config data
fastymllint -f yamllint file.yaml     # yamllint's exact output format
fastymllint -f json file.yaml
fastymllint -s --no-warnings dir/     # strict mode, errors only
fastymllint --list-files dir/
fastymllint -j 4 dir/                 # limit parallelism (default: all CPUs)

# Fix & format
fastymllint fix file.yaml             # apply safe fixes
fastymllint fix --unsafe file.yaml    # also apply semantic-affecting fixes
fastymllint fix --dry-run file.yaml   # show diff; exit 3 if changes needed
fastymllint format dir/               # normalize style (safe fixes only)
fastymllint format --dry-run dir/
```

### Exit codes

| code | meaning                                                       |
|------|---------------------------------------------------------------|
| 0    | success — no errors (lint), or fixes applied (fix/format)     |
| 1    | lint problems at error level (same as yamllint)               |
| 2    | warnings in `--strict` mode (same as yamllint); usage errors  |
| 3    | `--dry-run`: fixing/formatting would change files             |
| 255  | software failure (bad config, I/O error — yamllint uses -1)   |

`fix` and `format` exit 0 when fixes were applied successfully; the
dry-run "changes required" code (3) is distinct from software failures
(255).

## Configuration

fastymllint reads the same configuration as yamllint:

- `.yamllint`, `.yamllint.yaml` or `.yamllint.yml` in the current directory
  or any parent (up to `$HOME`), the `YAMLLINT_CONFIG_FILE` environment
  variable, or `~/.config/yamllint/config`;
- `extends: default | relaxed | <path>`, per-rule `level`
  (`error`/`warning`), `ignore` / `ignore-from-file` (gitignore-style
  patterns, also per rule), `yaml-files` patterns;
- all 23 yamllint rules with all their options: `anchors`, `braces`,
  `brackets`, `colons`, `commas`, `comments`, `comments-indentation`,
  `document-end`, `document-start`, `empty-lines`, `empty-values`,
  `float-values`, `hyphens`, `indentation`, `key-duplicates`,
  `key-ordering`, `line-length`, `new-line-at-end-of-file`, `new-lines`,
  `octal-values`, `quoted-strings`, `trailing-spaces`, `truthy`;
- inline directives: `# yamllint disable`, `# yamllint enable`,
  `# yamllint disable-line`, `# yamllint disable-file` (with
  `rule:<id>` arguments).

Known divergence from yamllint: the `locale` option is accepted but
ignored (`key-ordering` always compares by code point, matching the default
C locale).

fastymllint is written in 100% safe Rust (`#![forbid(unsafe_code)]`); the
YAML scanner, parser and config loader are its own — no C-derived YAML
library is involved.

## Development

```console
# Build
cargo build --release

# Set up the reference yamllint for parity tests and benchmarks
python3 -m venv .venv && .venv/bin/pip install yamllint

# Tests (unit + integration + side-by-side parity with yamllint)
cargo test

# Quick parity sweep over the example corpus
./tests/parity.sh

# Benchmarks vs yamllint
./bench/bench.sh
```

## Verified parity with yamllint

The parity suite (`tests/parity.rs`, `tests/parity.sh`) runs the real
yamllint and fastymllint side by side on the same files and asserts
**byte-for-byte identical stdout** (in yamllint's output format) **and
identical exit codes**. The corpus covers all 23 rules plus edge cases
(block scalars, flow nesting, anchors/tags, multi-documents, BOM/CRLF
files, inline disable directives, deliberate syntax errors), crossed with
several configurations (`default`, `relaxed`, custom rule options,
`--strict`, `--no-warnings`), as well as stdin, `ignore` patterns and
`--list-files`.

Current result: **208/208 comparisons matched — 100% identical output**.

One level deeper, a token-stream differential test (`dump_tokens` vs
`tests/dump_tokens.py`) asserts that the scanner produces the exact same
tokens as PyYAML — kinds, values, styles, and character-exact
line/column/pointer marks — across the whole corpus.

## Performance

`./bench/bench.sh` compares both tools on the same inputs. Representative
results (Apple Silicon, yamllint 1.38.0, average of 5 runs including
process startup):

| scenario                        | yamllint  | fastymllint | speedup |
|---------------------------------|-----------|-------------|---------|
| single small file (14 lines)    | 94.9 ms   | 32.5 ms     | ~3×     |
| single large file (25k lines)   | 2426.6 ms | 274.1 ms    | ~9×     |
| many files (400 files)          | 227.1 ms  | 36.4 ms     | ~6×     |

The small-file case is dominated by fixed process-startup cost on both
sides (fastymllint's own CPU time there is ~10 ms); the large-file case
reflects raw linting throughput, and the many-files case additionally
benefits from parallel processing.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for an overview of the
code.

## License and acknowledgements

fastymllint is based on the work of [yamllint](https://github.com/adrienverge/yamllint)
by Adrien Vergé and contributors: the linting pipeline, the 23 rules, their
options and messages, and the configuration system are reimplementations of
yamllint's behavior, developed with yamllint as the reference. yamllint is
released under the GNU General Public License v3, and fastymllint therefore
keeps the same license: **GPL-3.0-or-later** (see [LICENSE](LICENSE)).

The token scanner and event parser are derived from
[PyYAML](https://github.com/yaml/pyyaml) (MIT license).
