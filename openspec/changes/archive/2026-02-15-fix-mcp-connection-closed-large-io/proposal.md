## Why

The MCP server intermittently closes connections with `McpError -32000: Connection closed` during large reads and writes, which breaks agent workflows and can cause partial progress loss. This needs to be fixed now so Flashgrep can support production-grade, high-volume MCP sessions reliably.

## What Changes

- Add robust large-payload handling for MCP read/write tool calls so big responses and replacements do not terminate the session.
- Add explicit request/response size guardrails and structured error behavior instead of abrupt connection closure.
- Add transport-level resilience improvements in MCP stdio/TCP handlers (safe serialization, flush/error handling, and recoverable per-request failures).
- Add regression tests that simulate large `read_code`, `get_slice`, and `write_code` payloads and verify the server remains connected.
- Update user-facing docs with operational limits and recommended chunked workflows for very large IO.

## Capabilities

### New Capabilities
- `mcp-large-payload-resilience`: Reliable handling of large MCP tool payloads with bounded behavior and recoverable errors.

### Modified Capabilities
- `mcp-server`: Strengthen transport/request handling requirements to prevent connection drops during heavy tool usage.
- `token-efficient-code-io`: Extend read/write behavior requirements for large-range operations and structured limit handling.

## Impact

- Affected code: `src/mcp/mod.rs`, `src/mcp/stdio.rs`, `src/mcp/code_io.rs`, and related serialization/error paths.
- Affected APIs: MCP tool contracts for `read_code`, `write_code`, and `get_slice` responses/errors.
- Affected quality gates: new large-payload regression tests and docs updates for size/continuation guidance.
