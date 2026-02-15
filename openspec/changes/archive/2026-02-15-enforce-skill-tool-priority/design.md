## Context

Flashgrep already injects bootstrap skill guidance, but agents can still ignore it and use native read/write/search tools by default. This reduces consistency, increases token cost, and weakens the expected index-first operating model. The change requires cross-cutting updates across bootstrap payload semantics, skill content structure, and MCP response metadata so policy intent is explicit and enforceable by consuming clients.

## Goals / Non-Goals

**Goals:**
- Define a strict Flashgrep-first policy contract that can be interpreted deterministically by agent runtimes.
- Ensure bootstrap responses expose machine-readable policy metadata (priority levels, fallback gates, compliance hints).
- Optimize `skills/SKILL.md` so it remains concise but stronger in control flow and tool routing behavior.
- Preserve compatibility for clients that currently consume bootstrap payloads.

**Non-Goals:**
- Attempting to override agent platform security or system-level policies.
- Eliminating all fallback behavior when Flashgrep tools are unavailable.
- Introducing unrelated search/index runtime features.

## Decisions

### Decision: Separate human guidance from machine policy metadata
Bootstrap payloads will include both compact human instructions and structured policy fields (`preferred_tools`, `fallback_rules`, `policy_strength`, `compliance_checks`).

- Rationale: many agent runtimes can enforce structured fields more reliably than prose alone.
- Alternative considered: prose-only reinforcement in skill docs. Rejected due to weak deterministic enforcement.

### Decision: Add explicit fallback gating model
Fallback to native tools must require explicit gate conditions (e.g., missing index, unsupported operation, tool failure class), not generic preference.

- Rationale: prevents silent drift to native tools.
- Alternative considered: soft recommendations. Rejected because they do not produce reliable behavior.

### Decision: Keep policy optimized for token efficiency
Skill content should use compact decision tables and strict route ordering while avoiding repeated examples.

- Rationale: preserves session token budget and improves repeated compliance.
- Alternative considered: verbose exhaustive playbooks. Rejected due to context bloat.

### Decision: Provide compliance observability hooks
Bootstrap response metadata will expose fields that let clients report/track policy application state.

- Rationale: supports debugging when agents deviate from intended routing.
- Alternative considered: no observability fields. Rejected because failures become opaque.

## Risks / Trade-offs

- [Policy too strict for edge cases] -> Mitigation: keep narrow fallback gates and typed exception reasons.
- [Client incompatibility with new fields] -> Mitigation: additive contract changes with backward-compatible defaults.
- [Over-optimization harms clarity] -> Mitigation: maintain a short but complete decision sequence with one canonical example per workflow.
- [Different agent runtimes interpret policy differently] -> Mitigation: define normative MUST-level behavior in specs and provide compatibility notes.

## Migration Plan

1. Add policy contract fields to bootstrap payload schema and response builders.
2. Update MCP bootstrap tool docs/schema to advertise policy metadata and enforcement intent.
3. Refactor `skills/SKILL.md` to strict tool-priority decision logic with gated fallback.
4. Add tests validating bootstrap metadata presence and stable semantics across aliases/transports.
5. Update README/agent docs with compliance and troubleshooting guidance.

Rollback strategy:
- Keep existing bootstrap status behavior and disable strict policy flags if client interoperability issues appear.

## Open Questions

- Should policy strength levels support graded modes (`advisory`, `strict`) in the same release?
- Should compliance telemetry be emitted only in debug mode or always present as lightweight metadata?
