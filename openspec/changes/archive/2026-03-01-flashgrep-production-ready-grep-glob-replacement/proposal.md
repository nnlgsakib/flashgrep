## Why

Flashgrep still falls short of being a practical drop-in replacement for `grep` and shell glob workflows, especially for edge-case compatibility, filesystem coverage, and day-to-day operational commands. Closing these gaps now enables wider adoption in CI, automation, and cross-platform developer environments without requiring fallback to legacy tools.

## What Changes

- Expand `grep` compatibility behavior to cover expected matching, exit codes, output formatting, and error handling semantics used in real scripts.
- Upgrade glob support to match common shell expectations, including recursive behavior, hidden-file handling, and cross-platform path normalization.
- Add first-class filesystem operations (file and directory create/remove/move/copy/list/stat primitives) with consistent behavior on Windows, macOS, and Linux.
- Harden production-readiness with reliability requirements, large-workspace performance expectations, and deterministic failure modes.
- Update skills and user-facing documentation so the new capabilities are discoverable and safe to use in automation.

## Capabilities

### New Capabilities
- `filesystem-operations`: Cross-platform file and directory operations with safe defaults, clear error contracts, and automation-friendly CLI behavior.

### Modified Capabilities
- `grep-compat-search`: Extend compatibility guarantees so Flashgrep can replace core `grep` usage patterns in scripts and tooling.
- `glob-query-enhancements`: Improve glob semantics and path handling to meet practical shell-level expectations across platforms.
- `cli-search-commands`: Align command UX and output behavior with replacement-tool expectations for production usage.
- `ai-agent-documentation`: Update docs and skill guidance so operators and agents can use the new functionality correctly.

## Impact

- Affects CLI command surface, search/matching engine behavior, glob parser/execution flow, and platform-specific filesystem adapters.
- Introduces or expands integration and compatibility test suites for `grep`-style and glob-driven workflows.
- Adds documentation and skill updates for new commands, migration notes, and production usage guardrails.
- May require refactoring of error modeling and output rendering to guarantee deterministic cross-platform behavior.
