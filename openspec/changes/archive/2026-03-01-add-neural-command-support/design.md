## Context

Flashgrep already supports fast indexed lexical search, symbol-aware ranking, and deterministic CLI output, but it does not support intent-level natural-language retrieval. This change introduces an embedded neural path that keeps the existing local-first model: all embeddings, indexes, and model artifacts stay under `.flashgrep/` and operate without external serving infrastructure after initial bootstrap.

Key constraints:
- Preserve existing lexical workflows and performance characteristics for non-neural commands.
- Keep deterministic, script-safe CLI behavior.
- Avoid remote runtime dependencies after the first local model download.

## Goals / Non-Goals

**Goals:**
- Add a neural query path that maps natural-language intent to relevant project paths/snippets.
- Support first-run download and local caching of `BAAI/bge-small-en-v1.5` under `.flashgrep/`.
- Extend indexing to generate and persist chunk embeddings used by neural retrieval.
- Integrate semantic ranking into existing query command surfaces with explicit mode selection.

**Non-Goals:**
- Replacing or removing existing lexical search, symbol search, or ranking behavior.
- Building a cloud-hosted vector service or remote inference pipeline.
- Implementing full conversational memory/agent orchestration beyond retrieval.

## Decisions

### Decision: Local embedded model lifecycle under `.flashgrep/model-cache/`
- **Why**: Keeps behavior deterministic, avoids runtime network dependency after bootstrap, and aligns with Flashgrep's local CLI-first usage.
- **Approach**: On first neural command/index run, check local model artifacts; if missing, download `BAAI/bge-small-en-v1.5`, verify integrity, and cache for reuse.
- **Alternative considered**: Require users to manually pre-install model artifacts.
- **Why not alternative**: Adds setup friction and weakens out-of-box usability.

### Decision: Dual indexing path (lexical + vector) with shared chunk boundaries
- **Why**: Reuse existing chunking semantics so lexical and neural results reference the same locations and metadata.
- **Approach**: Generate embeddings for indexable chunks during indexing (initial and incremental), persist vector references alongside file/chunk metadata.
- **Alternative considered**: Separate semantic chunking strategy.
- **Why not alternative**: Increases complexity and risks divergent result coordinates.

### Decision: Explicit neural mode in CLI query flows
- **Why**: Prevents surprising behavior changes for existing scripts and keeps command semantics clear.
- **Approach**: Add a neural/semantic mode flag (or dedicated subcommand) that returns the same stable location-focused shape (path + line range + score/context metadata).
- **Alternative considered**: Implicitly blend lexical and neural retrieval by default.
- **Why not alternative**: Harder to reason about ranking behavior and regression impacts.

### Decision: Lightweight hybrid ranking for neural results
- **Why**: Pure embedding similarity can miss practical code navigation cues.
- **Approach**: Combine embedding similarity with existing structural metadata signals (path depth, recency, symbol presence when available) using deterministic weighting.
- **Alternative considered**: Similarity-only ranking.
- **Why not alternative**: Lower practical relevance for developer-oriented code search.

## Risks / Trade-offs

- [Model download failures or offline first run] -> Mitigation: clear error messaging, retry guidance, optional prefetch command, and no impact to lexical commands.
- [Indexing time/storage growth from embeddings] -> Mitigation: configurable embedding generation scope, batched generation, and compact on-disk vector format.
- [Cross-platform runtime differences for embedded inference] -> Mitigation: startup capability checks and graceful fallback with actionable diagnostics.
- [Ranking ambiguity between lexical and neural modes] -> Mitigation: explicit mode controls and mode-specific scoring fields in output.

## Migration Plan

1. Add model cache layout under `.flashgrep/` and bootstrap/download logic.
2. Extend index schema/storage to include vector metadata and embedding references.
3. Update index command to generate embeddings for chunk data and keep incremental updates consistent.
4. Add CLI neural mode and output schema updates.
5. Roll out with lexical path unchanged as default.

Rollback strategy:
- Disable neural mode path behind command/config guard while retaining lexical index/query behavior.
- Preserve backward-compatible reading of existing non-vector index artifacts.

## Open Questions

- Should model bootstrap happen during `index` by default, or only on first neural query invocation?
- Do we need an explicit `model pull`/`model doctor` CLI command for CI and air-gapped environments?
- What default max embedding batch size keeps memory stable on lower-end developer machines?
