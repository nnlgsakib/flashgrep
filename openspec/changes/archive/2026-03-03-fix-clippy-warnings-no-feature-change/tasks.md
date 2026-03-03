## 1. Baseline and lint-category planning

- [x] 1.1 Capture and group current clippy failures by category (`ptr_arg`, iterator/style, string length, test-only lints)
- [x] 1.2 Map each lint category to impacted modules and define behavior-parity checks per area

## 2. Core code lint remediation

- [x] 2.1 Replace `&PathBuf` argument usage with `&Path` where ownership is not required and update call sites
- [x] 2.2 Apply idiomatic control-flow and helper replacements (`strip_prefix`, `is_some_and`/`is_ok_and`, for-loop iterator updates)
- [x] 2.3 Remove redundant closures/returns and needless `as_bytes()` length calls while preserving logic

## 3. Test-target lint remediation

- [x] 3.1 Fix test-only clippy diagnostics (for example cloned-ref slice usage and `Default` field reassignment patterns)
- [x] 3.2 Re-run targeted tests around touched modules to confirm behavior parity after test-side refactors

## 4. Verification and completion

- [x] 4.1 Run `cargo test` (or equivalent targeted suites) and resolve regressions without changing feature behavior
- [x] 4.2 Run `cargo clippy --all-targets --all-features -- -D warnings` until clean
- [x] 4.3 Document final verification evidence and touched areas in the change notes
