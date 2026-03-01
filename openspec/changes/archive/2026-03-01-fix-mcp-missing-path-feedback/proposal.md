## Why

When an MCP tool is asked to read a missing file or list a missing folder, the agent does not always get clear machine-readable feedback that the target does not exist. This causes confusion, retry loops, and incorrect follow-up actions in automated workflows.

## What Changes

- Standardize MCP not-found error handling so missing files and directories return explicit, typed error payloads.
- Ensure all relevant MCP tools (read/slice/glob and related filesystem-targeting paths) emit deterministic not-found diagnostics.
- Add full MCP filesystem tool coverage for create/read/write/list/stat/copy/move/remove operations with machine-readable responses.
- Define deterministic overwrite, recursive, force, and dry-run semantics for MCP filesystem mutations.
- Align error envelope structure so agents can reliably detect and branch on not-found conditions.
- Update skill guidance so agents prefer Flashgrep MCP filesystem tools over native read/write/glob/grep workflows.
- Add test coverage for missing file, missing directory, invalid path, and MCP filesystem operation edge cases.

## Capabilities

### New Capabilities
- `mcp-not-found-feedback`: Consistent typed MCP feedback for missing file and directory paths.
- `mcp-filesystem-tools`: Full filesystem lifecycle operations exposed through MCP with deterministic automation-safe behavior.

### Modified Capabilities
- `mcp-server`: Tighten error contract for tool calls that operate on filesystem paths so not-found cases are explicit and machine-readable.
- `token-efficient-code-io`: Ensure `read_code`/`get_slice` path failures expose deterministic not-found metadata.
- `glob-query-enhancements`: Ensure missing root path handling returns clear typed feedback for agents.
- `ai-agent-documentation`: Update skill routing guidance so agents use MCP filesystem tools and not native read/write/glob fallbacks unless gated.

## Impact

- Affects MCP request handlers and error serialization paths in stdio/TCP flows.
- Adds MCP filesystem tool surface area and schema definitions for file and directory operations.
- May require harmonizing `invalid_params` and tool-level `isError` payload shapes.
- Adds integration tests for missing file/folder behavior, filesystem operation behavior, and regression checks for existing MCP clients.
