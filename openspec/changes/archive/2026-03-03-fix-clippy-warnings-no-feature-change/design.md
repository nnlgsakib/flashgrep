## Context

Running `cargo clippy --all-targets --all-features -- -D warnings` currently fails across production and test code. The diagnostics are primarily idiomatic and type-signature issues (`&PathBuf` arguments, manual pattern handling, iterator style, and redundant closures), but they span many modules and touch CLI, indexing, MCP, neural, DB, and watcher code.

The change must keep all current behavior intact. This is a quality and maintainability pass, not a feature release: CLI UX, MCP wire format, indexing/search semantics, and test expectations must remain stable.

## Goals / Non-Goals

**Goals:**
- Make strict clippy pass for all targets and all features.
- Refactor code in an idiomatic way that preserves observable behavior.
- Keep changes reviewable by applying mechanical, localized edits and validating parity with existing tests and command-level checks.

**Non-Goals:**
- No new user-facing features or command-line flags.
- No protocol/schema changes in MCP responses.
- No broad architecture rewrite or dependency replacement.

## Decisions

1. Standardize path-accepting APIs on `&Path` where ownership is not required.
   - **Why:** Removes repeated `clippy::ptr_arg` diagnostics and improves API flexibility without semantic changes.
   - **Alternative considered:** Keep `&PathBuf` and add `#[allow]` attributes. Rejected because it weakens lint guarantees and accumulates technical debt.

2. Prefer direct idiomatic constructs required by clippy (for-loops over `while let` iterators, `strip_prefix`, `is_some_and`/`is_ok_and`, direct function pointers in `map_err`).
   - **Why:** These are no-op behavior changes with clearer intent and lower maintenance risk.
   - **Alternative considered:** Targeted lint allowances. Rejected to keep `-D warnings` meaningful.

3. Keep lint fixes behavior-preserving and validate with command-level checks.
   - **Why:** Cross-cutting edits can introduce subtle regressions if not guarded.
   - **Alternative considered:** Module-by-module rollouts over multiple changes. Rejected for longer integration time and repeated CI churn.

4. Treat test-only clippy errors as first-class and fix them in place.
   - **Why:** The target command includes test targets; ignoring them leaves the repository non-compliant.
   - **Alternative considered:** Disable test target linting in CI. Rejected because it diverges from requested quality gate.

## Risks / Trade-offs

- [Signature updates fan out through call sites] -> Apply small, compile-verified batches and re-run clippy frequently.
- [Behavior drift from mechanical refactors] -> Preserve existing branching logic and add parity checks around touched command/protocol paths.
- [Large diff size across modules] -> Group edits by lint category (path args, iterator idioms, byte-length checks) to keep review coherent.
- [Runtime edge-case differences in string/path handling] -> Prefer equivalent standard-library transformations and keep existing tests green.

## Migration Plan

1. Capture current failing clippy diagnostics as baseline.
2. Apply fixes category-by-category across affected modules.
3. Run `cargo test` and existing smoke checks after each category.
4. Re-run `cargo clippy --all-targets --all-features -- -D warnings` until fully clean.
5. Land as a non-breaking internal quality update.

Rollback strategy: revert the change set if any behavior regressions are detected; no data migrations or persistent format updates are involved.

## Open Questions

- None currently; requirements are clear and bounded to lint compliance with behavior parity.

## Verification Evidence

- `cargo clippy --all-targets --all-features -- -D warnings` passes cleanly.
- `cargo test` passes (unit, integration, MCP integration, and doctests).
- Touched areas include chunking, CLI path plumbing, config/path helpers, DB interfaces, index scanner/engine/state, MCP code I/O and stdio/TCP handlers, neural startup/cache tests, search helpers, watcher path handling, and integration test determinism for model-cache state.
