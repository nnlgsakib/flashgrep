## Context

File discovery currently requires multiple steps to express common filtering needs (extension filtering, include/exclude paths, depth bounds, hidden-file behavior, and stable sorting). This produces extra calls and can increase latency in large repositories. The glob tool should become a one-pass discovery primitive for both users and agents.

## Goals / Non-Goals

**Goals:**
- Provide a comprehensive glob contract supporting include/exclude filters, extension constraints, recursion/depth controls, and hidden-file behavior.
- Add deterministic sorting/limiting controls so result sets are predictable and efficient.
- Preserve backward compatibility for existing glob usage by keeping current defaults.
- Keep performance strong on large codebases by minimizing unnecessary filesystem traversal.

**Non-Goals:**
- Replacing indexed search tools (`query`, `symbol`) with glob functionality.
- Introducing language-aware semantic filtering in this change.
- Building remote/distributed file discovery behavior.

## Decisions

### Decision 1: Expand glob input schema with composable filters
Add optional fields for `include`, `exclude`, `extensions`, `max_depth`, `follow_symlinks`, `include_hidden`, `case_sensitive`, and `limit` so callers can express most discovery intents in one call.

Alternative considered: multiple specialized glob methods. Rejected due to fragmented UX and duplicated implementation.

### Decision 2: Keep defaults backward-compatible
If new options are omitted, behavior mirrors current glob semantics.

Alternative considered: switching defaults to stricter filtering. Rejected because it risks silent behavior changes for existing clients.

### Decision 3: Deterministic result ordering
Support explicit `sort_by` and `sort_order` with stable tie-breaking so repeated calls produce consistent outputs.

Alternative considered: filesystem-native order only. Rejected because it is platform-dependent and less predictable.

### Decision 4: Early-prune traversal for performance
Apply include/exclude/depth constraints during traversal (not only post-filtering) to reduce IO and memory use.

Alternative considered: collect-all then filter. Rejected because it scales poorly in large repositories.

## Risks / Trade-offs

- [Overly broad filter combinations increase complexity] -> Mitigation: validate mutually incompatible options and return clear parameter errors.
- [Different OS path semantics can cause mismatch] -> Mitigation: normalize path separators and document matching behavior.
- [Extra options can increase user confusion] -> Mitigation: document canonical usage patterns and defaults in README/skill docs.
- [Sorting and filtering may add CPU overhead] -> Mitigation: prune early and short-circuit once `limit` is satisfied where safe.

## Migration Plan

1. Extend glob request schema and parser while preserving existing fields/defaults.
2. Implement traversal-time filtering and deterministic ordering.
3. Update MCP docs and skill guidance with examples for advanced glob usage.
4. Add regression/performance tests for common and worst-case filter combinations.
5. Roll out as additive behavior; existing calls continue to work unchanged.

Rollback strategy: disable new option parsing and fall back to legacy glob behavior.

## Open Questions

- Should extension matching include dot-prefixed forms only (`.rs`) or both (`rs`, `.rs`)?
- Should `case_sensitive` default vary by platform, or remain explicit/constant?
- Do we need an optional `return_metadata` mode (size/mtime/type) in this same change or later?
