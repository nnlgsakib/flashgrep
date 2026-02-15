## Context

Flashgrep currently applies payload safety limits that prevent hard crashes, but users now need effectively unbounded operation for very large text reads/writes and large-repo grep/glob traversal without losing precision. The challenge is to satisfy "no practical limit" expectations while preserving transport stability and deterministic behavior across CLI and MCP surfaces.

## Goals / Non-Goals

**Goals:**
- Support arbitrarily large logical operations by automatic chunking and continuation instead of single-response hard ceilings.
- Preserve exactness: line-accurate reads/writes, deterministic traversal/order, and no silent data loss.
- Keep MCP sessions stable under extreme payload workloads.
- Align grep/glob/read/write behavior between CLI and MCP.

**Non-Goals:**
- Building a new binary streaming protocol outside current JSON-RPC transport.
- Removing all safety controls at the transport level.
- Altering unrelated indexing architecture.

## Decisions

### Decision: Replace single-payload assumptions with resumable operation contracts
Large operations are expressed as multi-part workflows with continuation cursors, sequence metadata, and completion flags.

- Rationale: provides unbounded logical throughput while keeping each message safe.
- Alternative considered: truly unlimited single-message payloads. Rejected due to transport/process instability.

### Decision: Add precision guarantees per chunk
Each chunk response/write acknowledgement includes deterministic boundaries (`start_line`, `end_line`, offsets, chunk index) and integrity metadata.

- Rationale: ensures exact reconstruction and safe retries.
- Alternative considered: best-effort chunking without strict boundaries. Rejected due to precision risk.

### Decision: Unify large-operation orchestration in shared MCP/CLI helpers
Introduce shared continuation planning for read/write/query/glob so both transports and CLI behave identically.

- Rationale: avoids drift and inconsistent edge-case behavior.
- Alternative considered: separate transport-specific chunk logic. Rejected because maintenance cost and inconsistency risk are high.

### Decision: Maintain safety limits only at packet level, not operation level
Keep bounded request/response packet sizes, but auto-paginate and continue until full logical operation completes unless caller stops.

- Rationale: matches user expectation of handling "any amount" while preserving system health.
- Alternative considered: fail once packet limit is reached. Rejected as functionally limiting for large workloads.

## Risks / Trade-offs

- [Long-running operations consume resources] -> Mitigation: explicit continuation checkpoints and cancellable iteration.
- [Client complexity increases] -> Mitigation: provide default auto-continue behavior and documented loop examples.
- [Duplicate/skip bugs between chunks] -> Mitigation: deterministic cursor math + regression tests for boundary transitions.
- [Compatibility with existing clients] -> Mitigation: preserve legacy behavior for small payloads and add backward-compatible fields.

## Migration Plan

1. Introduce shared continuation model and chunk metadata types.
2. Refactor `read_code`, `get_slice`, and `write_code` to support auto-paginated logical operations.
3. Extend query/glob paths for large result sets with deterministic continuation windows.
4. Expose CLI options for automatic continuation and complete-result mode.
5. Add cross-transport and large-scale precision regression tests.
6. Update README and skill docs with large-operation usage patterns.

Rollback strategy:
- Keep packet-level safety helpers; disable auto-continue path and revert to prior bounded behavior if regressions appear.

## Open Questions

- Should auto-continue be enabled by default for all MCP calls, or opt-in per request?
- Should large write flows support transactional multi-chunk commit semantics in this change or a follow-up?
