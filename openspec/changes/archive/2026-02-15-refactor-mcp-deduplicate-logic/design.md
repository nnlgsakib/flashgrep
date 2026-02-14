## Context

The current MCP implementation duplicates similar logic across transport-specific paths (`src/mcp/mod.rs` for TCP and `src/mcp/stdio.rs` for stdio), plus separate metadata/definition logic in `src/mcp/skill.rs` and `src/mcp/tools.rs`. Repetition is highest around tool alias normalization, bootstrap handling, search-tool wiring, and response shaping. This increases bug risk when updating one transport and forgetting the other.

## Goals / Non-Goals

**Goals:**
- Centralize shared MCP operation logic used by both TCP and stdio handlers.
- Remove duplicate alias parsing and bootstrap payload generation while preserving current response contracts.
- Ensure logically equivalent methods return semantically equivalent payloads across transports.
- Keep refactor low-risk: no intentional behavior breaking changes for existing methods.

**Non-Goals:**
- Replacing JSON-RPC transport behavior or protocol framing.
- Redesigning tool semantics or renaming existing public methods.
- Large-scale architecture migration outside MCP module scope.

## Decisions

### Decision 1: Extract shared method handlers
Move repeated method logic (bootstrap trigger normalization, skill file loading, search helper behavior) into shared internal helpers callable by both TCP and stdio dispatchers.

Alternative considered: keeping duplicate implementations with stricter review rules. Rejected because drift has already occurred and review-only controls are fragile.

### Decision 2: Preserve response contracts and add consistency checks
Keep existing tool names and response shapes, but enforce consistency through tests that exercise both dispatch paths for key methods.

Alternative considered: normalizing all responses to a new schema immediately. Rejected due to compatibility risk for current clients.

### Decision 3: Consolidate tool metadata generation
Use `src/mcp/tools.rs` as shared source for tool definitions and aliases; reduce hard-coded duplicated tool list fragments in transport handlers.

Alternative considered: transport-specific metadata lists. Rejected due to repeated maintenance burden.

## Risks / Trade-offs

- [Hidden coupling in existing handlers] -> Mitigation: refactor incrementally and keep regression tests green after each extraction.
- [Accidental response shape changes] -> Mitigation: add assertions for key JSON fields and alias behavior in tests.
- [Over-abstraction making code harder to debug] -> Mitigation: keep helpers focused and avoid deeply generic layers.

## Migration Plan

1. Identify duplicated logic clusters in `mod.rs` and `stdio.rs`.
2. Extract shared helpers into MCP-internal module/functions with transport-agnostic signatures.
3. Rewire both transports to call shared helpers.
4. Update/expand tests for alias normalization, bootstrap states, and cross-transport consistency.
5. Run full suite and verify no behavior regressions.

Rollback strategy: revert to prior transport-specific handlers if regressions appear.

## Open Questions

- Should search helper parity include byte-for-byte response equality or semantic equality only?
- Do we want a follow-up change to unify stdio envelope formatting with TCP result style?
