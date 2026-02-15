## Why

Flashgrep already provides fast indexed search, but teams still need to keep `grep` and filesystem `glob` tools around for workflows not fully covered by current commands and APIs. Closing this parity gap now enables Flashgrep to be adopted as a production-ready single replacement for repeated search and discovery operations in developer and agent workflows.

## What Changes

- Add production-ready grep-parity capabilities to indexed text search, including regex support, case handling, file scoping, and context options with deterministic output limits.
- Add production-ready glob-parity capabilities to file discovery, including richer include/exclude behavior, extension filtering, hidden/symlink handling, deterministic sorting, and predictable pagination/limits.
- Define explicit compatibility and behavior guarantees for CLI and MCP usage so Flashgrep can be used as a complete replacement in automation.
- Add documentation updates in `README.md` describing grep/glob replacement workflows, migration guidance, and production usage examples.
- Optimize `skills/SKILL.md` for lower-token guidance while preserving correct Flashgrep-first tool usage and fallback rules.

## Capabilities

### New Capabilities
- `grep-compat-search`: Complete grep-style search behavior over indexed data with predictable, script-safe outputs for production automation.

### Modified Capabilities
- `glob-query-enhancements`: Expand existing glob behavior to full replacement-grade semantics and stricter determinism/performance guarantees.
- `cli-search-commands`: Extend CLI surface and command semantics to expose grep/glob parity options consistently.
- `mcp-server`: Ensure MCP tool contracts expose the new grep/glob parity options and stable response shapes.
- `ai-agent-documentation`: Update README and agent skill guidance for migration and low-token operational usage.

## Impact

- Affected code: CLI parsing/rendering, query/glob engines, index-backed filtering paths, MCP tool handlers, and docs assets.
- Affected interfaces: `flashgrep query`, `flashgrep files`/`glob`, and MCP `query`/`glob` methods (and related search helpers as needed).
- Operational impact: stronger production-readiness expectations around determinism, performance under large repos, and automation-safe outputs.
