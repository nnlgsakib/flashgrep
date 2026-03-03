## Context

This change removes neural/semantic search and AI navigation policy behavior from a codebase that currently supports lexical, semantic, and hybrid retrieval paths. The repository has cross-cutting neural references in CLI query parsing, search execution, startup prompting, MCP tool metadata, and policy/documentation specs. The requested outcome is a lexical/programmatic-only product with existing non-AI features preserved.

## Goals / Non-Goals

**Goals:**
- Remove runtime and build-time neural model dependencies and vector retrieval code paths.
- Remove semantic/hybrid retrieval modes from CLI and MCP contracts.
- Replace neural-first agent routing policy with deterministic programmatic routing guidance.
- Preserve behavior for indexing, lexical query execution, filesystem tools, symbol detection, metadata store, and watcher functionality.

**Non-Goals:**
- Reworking lexical ranking algorithms beyond changes needed to detach neural/hybrid blending.
- Introducing new search modes, new index storage engines, or new query DSL features.
- Changing unrelated CLI command groups or MCP filesystem lifecycle semantics.

## Decisions

1. Remove neural functionality end-to-end instead of feature-flag hiding.
   - Rationale: User intent is complete removal of AI-based navigation support, not optional disablement.
   - Alternatives considered:
     - Keep neural code behind a disabled default feature: rejected because maintenance and policy complexity remain.
     - Keep semantic mode but route to lexical internally: rejected because it preserves misleading API surface.

2. Treat semantic/hybrid mode removal as explicit breaking contract updates.
   - Rationale: Existing clients may still pass `retrieval_mode=semantic|hybrid`; behavior must be deterministic and documented.
   - Alternatives considered:
     - Silent coercion to lexical: rejected due to hidden behavior change.
     - Deprecated grace period in this change: rejected to keep scope aligned with full removal request.

3. Keep non-AI components unchanged and guard with focused regression checks.
   - Rationale: Requirement explicitly asks to keep other features intact.
   - Alternatives considered:
     - Broader cleanup/refactor while touching search stack: rejected due to increased regression risk.

## Risks / Trade-offs

- [Risk] Existing automation relying on semantic/hybrid modes will fail after upgrade. -> Mitigation: clear migration text in specs/docs and deterministic invalid-mode errors.
- [Risk] Removing neural modules may accidentally break shared search plumbing. -> Mitigation: preserve lexical interfaces and add/update tests around query parsing and result generation.
- [Trade-off] Short-term downstream migration cost for clients. -> Mitigation: provide explicit compatibility guidance and replacement examples.

## Migration Plan

1. Update specs and docs to remove neural-first and semantic/hybrid contracts.
2. Remove neural modules/dependencies and related startup prompt/model-cache behavior.
3. Update CLI and MCP argument schemas to lexical-only retrieval options.
4. Run regression validation for non-AI commands/features and targeted query-mode failure cases.
5. Rollback strategy: restore prior release/commit that still contains neural capability if consumers cannot migrate immediately.

## Open Questions

- Should removed semantic/hybrid mode values return a hard validation error or be omitted from schemas while preserving old error shapes?
- Are there any external scripts that parse bootstrap policy metadata fields specific to neural-first routing and require coordinated release notes?
