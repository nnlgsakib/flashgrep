## Context

`skills/SKILL.md` currently encodes routing policy, tool ordering, fallback gates, and workflow guidance in verbose prose. While functionally correct, it consumes unnecessary tokens during bootstrap and repeated agent planning. The requested change is to preserve existing behavior but re-express guidance in a compact structured language that is easier for agents to parse and cheaper to transmit.

## Goals / Non-Goals

**Goals:**
- Define a compact DSL-like format for `skills/SKILL.md` that keeps the same policy semantics.
- Preserve neural-first discovery behavior with deterministic lexical fallback.
- Preserve all fallback reason codes, compliance rules, and native-tool restrictions.
- Keep examples actionable for common flows (discovery, exact match, edit workflow) using structured blocks.

**Non-Goals:**
- Changing runtime CLI/MCP functionality.
- Changing policy metadata fields or bootstrap transport behavior.
- Introducing a new parser in runtime code (this is documentation format, not executable syntax).

## Decisions

1. Use block-based structured directives instead of prose sections.
   - Decision: represent guidance with short commands like `TASK`, `MODE`, `TOOLS`, `FALLBACK`, `RULE`, `EXAMPLE`, and edit snippets (`FILE`, `FIND`, `REPLACE`).
   - Rationale: predictable shape lowers token and parsing cost while keeping intent explicit.
   - Alternatives considered:
     - Keep markdown prose and trim wording only: rejected (still high token overhead and variable phrasing).
     - Move guidance into JSON schema only: rejected (less human-readable and harder to maintain manually).

2. Keep behavior parity as a hard requirement.
   - Decision: include explicit parity checks for tool order, fallback gates, and compliance recovery semantics.
   - Rationale: user asked for same features with less token usage, not policy changes.
   - Alternatives considered:
     - Simplify by removing some edge-case guidance: rejected due to behavior drift risk.

3. Preserve compatibility references in README.
   - Decision: README will mention the structured-skill format and show one concise example mapping.
   - Rationale: users and agent authors need to understand the new compact convention quickly.

## Risks / Trade-offs

- [Risk] Ambiguity in new DSL keywords causes inconsistent agent interpretation. -> Mitigation: define a fixed keyword set with concise semantics and examples.
- [Risk] Omitting a fallback gate during rewrite changes behavior. -> Mitigation: checklist-based parity verification against existing `skills/SKILL.md` sections before merge.
- [Trade-off] Very compact format may reduce readability for first-time humans. -> Mitigation: include a short legend and 2-3 concrete examples.

## Migration Plan

1. Draft structured-skill DSL format and map old sections to new keywords.
2. Rewrite `skills/SKILL.md` in DSL form preserving all behavioral contracts.
3. Update README references to reflect the compact format and usage examples.
4. Verify parity checklist (tool order, neural-first fallback flow, gate list, compliance recovery, native tool bans).
5. Rollback plan: restore previous `skills/SKILL.md` from git history if any integration regressions are reported.

## Open Questions

- Should we standardize the DSL keyword vocabulary as a reusable template for other skills beyond Flashgrep?
- Do we want an optional compact+expanded dual view (`SKILL.md` + `SKILL.expanded.md`) for human onboarding?
