## Context

Flashgrep already injects agent guidance and tool-priority metadata through bootstrap, but the current policy is Flashgrep-first at a broad tool-family level and does not explicitly prioritize neural retrieval over programmatic search for agent discovery tasks. Agents therefore often start with lexical/programmatic workflows and only later switch to semantic retrieval, increasing iteration count.

This change introduces neural-first routing for agent search intent while preserving deterministic fallback to programmatic search when neural preconditions are not met.

## Goals / Non-Goals

**Goals:**
- Encode a deterministic neural-first routing policy in bootstrap metadata and guidance payloads.
- Define explicit fallback gates/reasons that allow programmatic search as second priority.
- Update agent documentation and skill guidance so neural-first behavior is operationally clear.
- Preserve deterministic observability fields so policy compliance can be inspected.

**Non-Goals:**
- Removing lexical/programmatic search capabilities.
- Changing low-level ranking algorithms for semantic or lexical query execution.
- Introducing autonomous tool chaining beyond policy/guidance and routing order.

## Decisions

### Decision: Policy should separate discovery intent from exact-match intent
- **Choice**: Neural-first applies to discovery/intent queries; exact-match operations remain eligible for lexical-first via explicit fallback gates.
- **Rationale**: Prevents loss of precision for exact literals while still improving default agent exploration.
- **Alternative considered**: Force neural-first for all query classes.
- **Why not**: Can degrade deterministic exact-match workflows.

### Decision: Bootstrap metadata includes ordered retrieval preference and gate reasons
- **Choice**: Add/extend machine-readable policy fields for ordered retrieval strategy (`semantic` then `lexical`) and typed reason codes.
- **Rationale**: Enables deterministic enforcement and debugging across clients.
- **Alternative considered**: Guidance-only prose with no machine-readable ordering.
- **Why not**: Hard to enforce and audit in automated agents.

### Decision: Programmatic fallback is explicitly gated
- **Choice**: Allow lexical/programmatic fallback only when one of a bounded set of reasons is present (model unavailable, low confidence, explicit exact/literal requirement, syntax/parse constraints).
- **Rationale**: Keeps behavior predictable and avoids silent policy drift.
- **Alternative considered**: Permit unrestricted fallback.
- **Why not**: Reintroduces inconsistent tool routing and higher token/tool churn.

## Risks / Trade-offs

- [Neural-first overuse in exact lookup tasks] -> Mitigation: include exact/literal fallback gate and documentation examples.
- [Policy complexity increases bootstrap payload size] -> Mitigation: compact, machine-readable fields and concise guidance text.
- [Client inconsistency in honoring order] -> Mitigation: add compliance metadata and troubleshooting guidance in docs.
- [Model-unavailable environments] -> Mitigation: deterministic fallback to programmatic mode with typed reason.

## Migration Plan

1. Update policy spec and bootstrap guidance contract to encode neural-first ordering and fallback gates.
2. Update docs/skill text to align with the new routing contract.
3. Implement metadata and guidance changes in bootstrap emitters.
4. Add tests for ordering and fallback reason code behavior.

Rollback strategy:
- Revert policy metadata order to previous Flashgrep-first generic ordering while keeping existing tool compatibility.

## Open Questions

- Should fallback due to low-confidence use a numeric threshold in policy metadata, or a qualitative reason only?
- Should clients be required to emit retrieval decision traces for each search action, or only on deviation?
