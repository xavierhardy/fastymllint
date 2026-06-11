# Contributing to fastymllint

Thank you for your interest in contributing to fastymllint!

## Getting Started

1. Clone the repository:

   ```bash
   git clone https://github.com/xhardy/fastymllint
   cd fastymllint
   ```

2. Build:

   ```bash
   cargo build --release
   ```

3. Set up the reference yamllint (needed by parity tests and benchmarks):

   ```bash
   python3 -m venv .venv
   .venv/bin/pip install yamllint
   ```

4. Run the tests:

   ```bash
   ./test.sh        # or: cargo test
   ```

## Ground rules

- **Parity first**: fastymllint must produce byte-identical output to
  yamllint (in `-f yamllint` format). Any change to the scanner, linter or
  rules must keep `cargo test` and `./tests/parity.sh` green. When fixing a
  parity bug, add the offending YAML to `tests/data/tricky/` so it stays
  fixed.
- The behavioral reference is the yamllint source installed in `.venv`
  (`.venv/lib/python*/site-packages/yamllint/`).
- Lint and format before submitting: `./lint-format.sh` (runs `cargo fmt`
  and `cargo clippy`).
- Add tests for new functionality (safe fixers must come with
  semantic-preservation checks).
- fastymllint is based on yamllint (GPL-3.0-or-later) and keeps the same
  license; by contributing you agree to license your contribution under
  GPL-3.0-or-later (see [LICENSE](LICENSE)).
