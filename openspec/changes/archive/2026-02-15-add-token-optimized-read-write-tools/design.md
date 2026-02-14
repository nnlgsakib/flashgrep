## Context

The current workflow is optimized for search/discovery but not for token-efficient code retrieval and edits. Agents frequently need small, precise context windows and safe, incremental writes, yet existing read/write patterns can return more content than needed or require broader retries after conflicts. This change introduces explicit, budget-aware read semantics and guarded minimal-diff write semantics so agent interactions remain efficient and reliable at scale.

## Goals / Non-Goals

**Goals:**
- Provide deterministic read operations with strict budgets (`max_tokens`, `max_bytes`, `max_lines`) and continuation support.
- Support targeted retrieval modes: path-based slices and symbol-based extraction with limited surrounding context.
- Provide minimal-diff writes constrained to explicit line ranges with optimistic concurrency preconditions.
- Return compact, machine-friendly metadata and conflict payloads that allow automatic retry/rebase strategies.
- Preserve correctness under concurrent edits while minimizing token overhead for both success and failure paths.

**Non-Goals:**
- Replacing existing search/index features or removing current tools.
- Building AST-level refactoring or semantic code transforms in this change.
- Guaranteeing zero-cost operations across all file types and encodings.

## Decisions

### Budgeted read contract
Read endpoints will accept explicit caps (`max_tokens`, `max_bytes`, `max_lines`) and apply deterministic truncation with `applied_limits` and `next_cursor` metadata.

Alternative considered: fixed server-side defaults without caller control. Rejected because agent workloads vary widely and caller-provided limits produce better token efficiency.

### Dual read modes: slice and symbol
Support both direct slice reads (`file_path`, `start_line`, `end_line`) and symbol-oriented reads (`symbol_name`, optional context lines). This avoids overfetching while still handling unknown line ranges.

Alternative considered: symbol-only reads. Rejected because line-range reads are essential for deterministic patches and retry flows.

### Minimal metadata levels
Responses include metadata profiles (`minimal`, `standard`) so callers can remove non-essential fields during high-volume runs.

Alternative considered: always returning full diagnostics. Rejected due to avoidable token overhead.

### Guarded minimal-diff writes
Write endpoint will require explicit line-range replacements and optional preconditions (file hash and/or expected boundary text). On mismatch, return structured conflict details rather than applying partial writes.

Alternative considered: whole-file blind overwrite. Rejected due to race risks and higher token costs from repeated full-file transfers.

## Risks / Trade-offs

- [Tighter budgets can under-fetch critical context] -> Mitigation: expose continuation cursor and lightweight hints for recommended follow-up reads.
- [Line-based write targeting can drift under heavy concurrent edits] -> Mitigation: enforce preconditions and return conflict metadata that pinpoints mismatch.
- [More options increase API complexity] -> Mitigation: document stable defaults and provide a minimal profile for common agent paths.
- [Token estimation variance by model] -> Mitigation: treat token caps as best-effort with deterministic byte/line backstops.

## Migration Plan

1. Add new read/write tool handlers and schemas behind the MCP surface.
2. Integrate with existing index/symbol lookup where available for symbol reads.
3. Add tests for budget limits, continuation, conflicts, and deterministic outputs.
4. Roll out as additive capability; keep existing workflows available.
5. Monitor performance and conflict rates; tune defaults without changing contracts.

Rollback: disable/withdraw new tool registrations while retaining existing tools; no persistent data migration required.

## Open Questions

- Should continuation cursors be opaque tokens only, or also expose line hints for debugging?
- What default `metadata_level` should be used by first-party agents (`minimal` vs `standard`)?
- Do we need a hard per-request cap policy to protect server resources under adversarial usage?
