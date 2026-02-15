## Context

Flashgrep MCP clients report intermittent `McpError -32000: Connection closed` during large `read_code`, `get_slice`, and `write_code` operations. Current transport handlers process request lines and payload serialization inline; when payload size or write pressure spikes, failures can surface as abrupt connection termination instead of structured tool-level errors. This change must preserve existing MCP method contracts while adding strong resilience for large IO paths.

## Goals / Non-Goals

**Goals:**
- Prevent transport-level connection drops for large read/write payload workflows.
- Enforce bounded request/response behavior with explicit limits and continuation guidance.
- Convert oversized/invalid large-IO cases into structured MCP errors instead of process/connection failure.
- Keep behavior consistent across stdio and TCP handlers.

**Non-Goals:**
- Introducing a streaming protocol beyond JSON-RPC line-delimited transport.
- Changing existing small-payload success shapes in incompatible ways.
- Adding unrelated query/glob feature work.

## Decisions

### Decision: Add explicit IO payload guardrails at MCP boundary
Add configurable upper bounds for request argument size, serialized response size, and replacement payload size for `write_code`.

- Rationale: turns crash-prone extreme payloads into deterministic rejections.
- Alternative considered: no hard limit and rely only on OS buffering. Rejected due to unpredictable failure under pressure.

### Decision: Standardize structured error envelopes for large IO failures
Return typed `invalid_params`/`payload_too_large` style response payloads with actionable fields (limit, observed, suggested continuation chunk).

- Rationale: clients can recover automatically instead of treating close as fatal unknown.
- Alternative considered: generic string error messages. Rejected due to weak machine-retry support.

### Decision: Centralize write/read serialization safety path
Use shared helper(s) for serializing tool responses and writing transport output with consistent error capture and flush handling.

- Rationale: avoids drift and inconsistent behavior between stdio and TCP handlers.
- Alternative considered: keep duplicated per-transport logic. Rejected due to prior divergence risk.

### Decision: Prefer chunked continuation for large reads
When read payload would exceed limits, truncate deterministically and include continuation metadata to request the next segment.

- Rationale: aligns with token-efficient patterns and keeps sessions alive.
- Alternative considered: return full payload then rely on client truncation. Rejected due to connection stability risk.

## Risks / Trade-offs

- [Overly strict limits reduce usability] -> Mitigation: set conservative defaults and document override/config points.
- [Behavior drift between stdio and TCP] -> Mitigation: shared helper coverage and cross-transport regression tests.
- [Breaking existing clients expecting raw error strings] -> Mitigation: preserve existing top-level fields and add structured fields compatibly.
- [Large write latency remains high despite no crash] -> Mitigation: enforce replacement-size guidance and recommend chunked writes.

## Migration Plan

1. Introduce limit constants/config plumbing for MCP request/response and write replacement payload sizes.
2. Add shared response serialization + transport write safety helpers.
3. Wire `read_code`, `get_slice`, and `write_code` paths to limit-aware behavior and structured errors.
4. Add cross-transport regression tests for large reads/writes and verify no connection drop.
5. Update README/skill guidance with large-IO operational recommendations.

Rollback strategy:
- If regressions occur, disable new strict limit enforcement while keeping stability helpers; revert to prior payload size behavior but retain improved error handling.

## Open Questions

- Should payload limits be user-configurable in `.flashgrep/config.json` immediately, or start as fixed defaults and expose config in a follow-up?
- Should server responses include a stable numeric error code for `payload_too_large` beyond textual typed error tags?
