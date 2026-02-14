## Context

Users report that paths listed in `.flashgrepignore` (for example `.opencode/`) still appear in indexed results. This indicates ignore matching is not consistently applied across all indexing/update paths. Additionally, ignore changes are currently passive: when users add new ignore patterns, previously indexed files under newly ignored paths remain in the index until a manual full clear/reindex.

This change spans scanner/indexing behavior, watcher behavior, and metadata/index pruning, so a cross-module design is needed.

## Goals / Non-Goals

**Goals:**
- Ensure `.flashgrepignore` patterns are enforced consistently for initial indexing and incremental updates.
- Ensure adding ignore patterns triggers live reconciliation that removes newly ignored files from index + metadata.
- Keep behavior deterministic and observable (clear logs and prune counts).

**Non-Goals:**
- No changes to MCP/CLI API shapes.
- No changes to core ranking or query semantics.
- No remote/shared index synchronization.

## Decisions

- **Decision: Canonical relative-path matching for ignore checks**
  - **Rationale:** Ignore patterns are repository-root semantics; matching should use normalized repo-relative paths to avoid platform/path separator inconsistencies.
  - **Alternative considered:** Matching raw absolute paths.
  - **Why not alternative:** Leads to inconsistent behavior across watcher/scanner code paths.

- **Decision: Trigger reconciliation on `.flashgrepignore` change events**
  - **Rationale:** Users expect immediate effect when ignore rules change.
  - **Alternative considered:** Apply new rules only on next full `index` run.
  - **Why not alternative:** Leaves stale ignored content visible and breaks user expectation.

- **Decision: Prune by file path set from metadata store, then remove text documents**
  - **Rationale:** Metadata is authoritative for indexed files and enables efficient affected-file discovery.
  - **Alternative considered:** Re-scan full repository and rebuild from scratch on each ignore change.
  - **Why not alternative:** Too expensive and unnecessary for incremental workflows.

- **Decision: Keep prune operation idempotent and bounded**
  - **Rationale:** Repeated ignore updates should not corrupt state; reruns should produce stable results.
  - **Alternative considered:** Best-effort deletions without consistency checks.
  - **Why not alternative:** Can leave index/metadata divergence.

## Risks / Trade-offs

- **[Risk]** Large ignore updates may trigger heavy prune operations  
  **Mitigation:** Batch deletions and report counts; keep operation incremental by affected paths only.

- **[Risk]** Path normalization bugs on Windows/Unix separators  
  **Mitigation:** Centralize normalization utility and reuse in scanner + watcher + prune paths.

- **[Risk]** Temporary index/metadata mismatch on partial failures  
  **Mitigation:** Execute prune in ordered steps with error reporting and retry-safe idempotency.

- **[Trade-off]** More watcher logic complexity  
  **Mitigation:** Isolate ignore-reload/reconcile logic into dedicated helper functions.

## Migration Plan

1. Implement unified ignore matching utility (repo-relative normalized paths).
2. Apply utility to initial scan and incremental file-event indexing paths.
3. Add watcher handling for `.flashgrepignore` updates.
4. Implement prune routine for newly ignored files (metadata + text index).
5. Add tests for ignore enforcement and live prune behavior.

Rollback strategy: disable ignore-change reconciliation trigger while keeping baseline indexing intact.

## Open Questions

- Should ignore reconciliation run synchronously on watcher event or be debounced/batched under heavy write bursts?
- Should prune logs include top-N affected paths or only aggregate counts by default?
