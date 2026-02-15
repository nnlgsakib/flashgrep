## 1. Payload Limits and Shared Safety Helpers

- [x] 1.1 Define MCP payload limit constants/config for request size, response size, and `write_code` replacement size.
- [x] 1.2 Add shared helper(s) for safe JSON serialization and transport writes with consistent flush/error handling.
- [x] 1.3 Add shared structured error builder for size-limit and invalid-parameter failures with machine-actionable metadata.

## 2. Read Path Resilience

- [x] 2.1 Update `read_code` to enforce safety bounds and return deterministic truncation/continuation when near payload limits.
- [x] 2.2 Update `get_slice` handling to reject oversized ranges with structured limit metadata instead of transport failure.
- [x] 2.3 Ensure read-path failures return recoverable MCP errors while keeping stdio/TCP sessions alive.

## 3. Write Path Resilience

- [x] 3.1 Enforce maximum replacement size in `write_code` before mutation and return structured `payload_too_large` style error details.
- [x] 3.2 Preserve existing precondition semantics while adding explicit oversized-write rejection behavior.
- [x] 3.3 Add guidance fields in oversized write responses (for example max allowed replacement size and chunking recommendation).

## 4. Transport Consistency Across MCP Handlers

- [x] 4.1 Refactor stdio MCP request handling to use shared per-request error isolation and avoid connection teardown on tool-level failures.
- [x] 4.2 Refactor TCP MCP request handling to mirror stdio resilience behavior for large IO errors.
- [x] 4.3 Add regression tests proving that after an oversized read/write failure, subsequent requests on the same session still succeed.

## 5. Verification and Documentation

- [x] 5.1 Add/extend tests for large `read_code`, `get_slice`, and `write_code` payload scenarios across both transports.
- [x] 5.2 Add compatibility tests confirming existing small-payload behavior remains unchanged.
- [x] 5.3 Update README and skill guidance with large-IO operational limits and chunked retry recommendations.
