# Cross-Platform Manual Validation Guide

Use this guide to complete the final production-readiness validation task on Linux, macOS, and Windows.

## Scope

Validate:

- grep-compatible query behavior
- glob parity behavior
- filesystem command behavior
- deterministic ordering/windowing
- full test pass on each platform

## Preconditions (all platforms)

1. Use the same commit on each machine.
2. Use Rust stable toolchain.
3. Run from repository root.

## 1) Build + baseline tests

```bash
cargo fmt --check
cargo test
```

Expected: all tests pass.

## 2) Query parity checks

```bash
# Match exists => exit 0
flashgrep query "main" --limit 5

# No match => exit 1
flashgrep query "__NO_MATCH_TOKEN__" --limit 5

# Fixed-string multi-pattern
flashgrep query "ignored" --fixed-string "alpha" --fixed-string "beta" --output json

# Regex + case-insensitive
flashgrep query "fn\\s+main" --mode regex --ignore-case --output json
```

Expected:

- deterministic output shape
- no-match command returns exit code `1`

## 3) Files/glob parity checks

```bash
flashgrep files --pattern "src/**/*.rs" --sort-by path --sort-order asc --offset 0 --limit 20 --output json
flashgrep files --pattern "src/*.[rl][si]" --sort-by path --sort-order asc --output json
flashgrep files --pattern "**/*" --include-hidden --sort-by path --sort-order asc --offset 0 --limit 50 --output json
flashgrep files --pattern "**/*" --sort-by path --sort-order asc --offset 50 --limit 50 --output json
```

Expected:

- stable sorted windows
- no duplicates/gaps when paging
- character class and hidden-path behavior matches docs

## 4) Filesystem operation checks

```bash
flashgrep fs create tmp/fg-validate --dir --parents
flashgrep fs create tmp/fg-validate/a.txt --parents
flashgrep fs stat tmp/fg-validate/a.txt --output json
flashgrep fs list tmp/fg-validate --sort-by path --sort-order asc --output json
flashgrep fs copy tmp/fg-validate/a.txt tmp/fg-validate/b.txt --overwrite
flashgrep fs move tmp/fg-validate/b.txt tmp/fg-validate/c.txt --dry-run
flashgrep fs remove tmp/fg-validate --recursive --dry-run
flashgrep fs remove tmp/fg-validate --recursive --force
```

Expected:

- conflict rules enforced unless `--overwrite`
- dry-run mutators do not change filesystem
- JSON responses include stable fields (`path`, `file_type`, `size`, `modified_unix`, `readonly`)

## 5) Platform path normalization check

Run includes/excludes with native separators and verify equivalent result sets.

- Windows example include: `src\\**\\*.rs`
- POSIX example include: `src/**/*.rs`

Expected: equivalent filtering behavior after normalization.

## Validation Record Template

Record one section per platform:

```text
Platform: <Windows|Linux|macOS>
Rust: <rustc --version>
Commit: <git rev-parse HEAD>

Build/Test:
- cargo fmt --check: PASS/FAIL
- cargo test: PASS/FAIL

Query parity: PASS/FAIL
Glob parity: PASS/FAIL
Filesystem ops: PASS/FAIL
Path normalization: PASS/FAIL

Notes:
- <issues or regressions>
```
