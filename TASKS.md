# Tasks

## Done (see docs/DONE_TASKS.md for history)

- [x] Rewrite as **fastymllint**: a drop-in yamllint replacement (YAML only)
- [x] Port PyYAML's scanner/parser to Rust with exact token marks
- [x] Port all 23 yamllint rules with byte-identical messages
- [x] yamllint-compatible configuration (extends, levels, ignore, yaml-files,
      inline directives, rule options)
- [x] Output formats: all yamllint formats (parsable, standard, colored,
      github, auto — the default) plus text and json extensions
- [x] Parallel file processing (rayon, `-j/--jobs`)
- [x] Auto-fix (`fix`, `fix --unsafe`) and auto-format (`format`) with
      `--dry-run` diffs and dedicated exit code
- [x] Side-by-side parity tests against yamllint + token-level differential
      tests against PyYAML
- [x] Benchmark harness (`bench/bench.sh`)

## Backlog

- [ ] `locale` config option (affects `key-ordering` collation in yamllint)
- [ ] Fixer for `key-duplicates` (delete duplicate entries; needs block
      extent analysis)
- [ ] Smarter `line-length` fixer (folding long scalars where safe)
- [ ] Publish to crates.io; prebuilt binaries
- [ ] Reduce token cloning in the linter pipeline (perf headroom)
