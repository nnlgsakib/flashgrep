## Why

The MCP implementation currently contains repetitive and partially duplicated logic across `src/mcp/mod.rs`, `src/mcp/stdio.rs`, `src/mcp/skill.rs`, and `src/mcp/tools.rs`. This duplication increases maintenance risk, makes behavior drift more likely, and slows future feature changes.

## What Changes

- Refactor MCP routing and bootstrap/skill handling to use shared internal helpers instead of repeated per-transport implementations.
- Consolidate repetitive tool definition and validation paths so equivalent tools behave consistently across stdio and TCP MCP handlers.
- Remove or simplify nonsensical/redundant logic paths while preserving existing tool contracts and response behavior.
- Add coverage to ensure no functional regressions during deduplication/refactor work.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `mcp-server`: Add explicit requirements for cross-transport behavioral consistency and centralized shared handler usage for equivalent MCP operations.

## Impact

- Affected code: `src/mcp/mod.rs`, `src/mcp/stdio.rs`, `src/mcp/skill.rs`, `src/mcp/tools.rs`, and related tests.
- No intended API breaking changes; MCP methods and existing behavior should remain compatible.
- Reduced maintenance overhead and lower risk of divergent logic between transports.
