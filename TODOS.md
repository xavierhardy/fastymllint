# TODOs

Genuine improvements identified by reviewing the code against the reference
yamllint (1.38.0, installed in `.venv`). Each item is implemented and
committed separately.

- [ ] **1. Output format CLI parity** (backlog item): support yamllint's
      `-f parsable|standard|colored|github|auto` in addition to
      `text`/`json`/`yamllint`, with byte-identical output:
      - `parsable` — same as fastymllint's `text` format;
      - `colored` — standard format with yamllint's exact ANSI escapes;
      - `github` — `::group::`/`::warning`/`::error`/`::endgroup::`
        workflow commands;
      - `auto` — `github` when `GITHUB_ACTIONS` + `GITHUB_WORKFLOW` are set,
        `colored` on a tty, `standard` otherwise — and make it the default,
        like yamllint (previously `text` was the default).
      Extend the parity suite to sweep `-f parsable` and `-f github`.
- [ ] **2. Usage-error parity for missing input**: `fastymllint --list-files`
      with no files currently exits 0; yamllint treats missing
      `FILE_OR_DIR`/`-` as a usage error (exit 2) even with `--list-files`.
- [ ] **3. I/O error message parity**: unreadable files print
      `[Errno 2] No such file or directory (os error 2): 'x.yaml'` with a
      hardcoded errno — wrong errno for non-ENOENT errors and a stray
      `(os error N)` suffix. Match Python's `OSError` string:
      `[Errno N] <strerror>: '<path>'`.
- [ ] **4. Reduce token cloning in the linter pipeline** (backlog item):
      `token_or_comment_or_line_generator` re-clones every token/comment
      element (each holding up to four owned `Token`s) when merging with
      lines; restructure the merge to consume the vector instead.
