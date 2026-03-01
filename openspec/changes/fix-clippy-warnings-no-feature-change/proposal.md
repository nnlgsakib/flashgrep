## Why

`cargo clippy --all-targets --all-features -- -D warnings` currently fails with dozens of diagnostics, which blocks quality-gated workflows and slows delivery. We need the codebase to pass strict linting while preserving all existing behavior and user-facing features.

## What Changes

- Fix all current clippy diagnostics across library code and tests so the strict command succeeds.
- Prefer idiomatic signatures and control-flow constructs (for example `&Path` over `&PathBuf`, iterator-based loops, and direct function references) where clippy requires them.
- Keep behavior unchanged: no feature removals, no CLI contract changes, and no API semantic changes.
- Add or update targeted verification to ensure lint-driven refactors do not alter runtime outcomes.

## Capabilities

### New Capabilities
- `clippy-clean-build`: The repository can be linted with all targets and all features enabled under `-D warnings` without changing functional behavior.

### Modified Capabilities
- `cli-interface`: Clarify that internal refactors for lint compliance must preserve existing CLI arguments, output, and error behavior.
- `mcp-server`: Clarify that lint-motivated code cleanups must not change MCP request and response semantics.

## Impact

- Affected code spans multiple Rust modules, including chunking, CLI wiring, config/path helpers, DB layer, indexing, MCP handlers, neural helpers, search internals, and watcher utilities.
- No new third-party dependencies are required.
- CI and developer workflows that enforce clippy become unblocked once complete.
