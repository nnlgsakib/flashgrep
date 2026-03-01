## Context

Flashgrep is a local-first Rust CLI optimized for deterministic, high-speed lexical search. Users now need natural-language lookup against repository knowledge (for example, asking for auth code locations) without introducing remote services. This change adds an embedded neural path that reuses existing chunk/index metadata and preserves script-safe CLI behavior.

Constraints:
- Existing lexical query behavior must remain default and backward compatible.
- Neural mode must remain deterministic enough for repeated automation on unchanged index/model state.
- Model download happens only on first use and remains cached in `.flashgrep/`.

## Goals / Non-Goals

**Goals:**
- Add CLI-accessible semantic retrieval (plus optional hybrid ranking) over repository vectors.
- Bootstrap and cache `BAAI/bge-small-en-v1.5` under `.flashgrep/model-cache/` on first neural use.
- Extend indexing to persist vector artifacts for chunk-level retrieval.
- Keep output shape stable and path-oriented across lexical and neural query modes.

**Non-Goals:**
- Replacing lexical/tantivy search as the primary default query path.
- Introducing hosted vector services or runtime cloud dependencies.
- Adding conversational memory/agent features beyond retrieval.

## Decisions

### Decision: Explicit retrieval mode in CLI
- **Choice**: Add explicit retrieval mode selection (`lexical`, `semantic`, `hybrid`) instead of silently changing query semantics.
- **Rationale**: Preserves existing script behavior and makes ranking behavior transparent.
- **Alternative considered**: Always blend lexical and semantic scoring.
- **Why not**: Harder to debug and more likely to surprise existing automation.

### Decision: Local model cache with first-use bootstrap
- **Choice**: Download `BAAI/bge-small-en-v1.5` assets once and cache in `.flashgrep/model-cache/`.
- **Rationale**: Keeps runtime local and deterministic after first bootstrap.
- **Alternative considered**: Require manual model install before use.
- **Why not**: Higher setup friction and poorer out-of-box UX.

### Decision: Chunk-aligned vectors in metadata store
- **Choice**: Generate embeddings for existing chunks and store references keyed by file path + line range + hash.
- **Rationale**: Aligns lexical and semantic results on identical coordinates.
- **Alternative considered**: Separate semantic chunking/index.
- **Why not**: More complexity and potential mismatch between lexical and semantic coordinates.

### Decision: Deterministic hybrid ranking
- **Choice**: Combine semantic similarity with lightweight deterministic signals (path depth, recency, lexical presence).
- **Rationale**: Improves practical code navigation relevance.
- **Alternative considered**: Similarity-only ranking.
- **Why not**: Often weaker ranking quality in large heterogeneous codebases.

## Risks / Trade-offs

- [First-run download fails or offline bootstrap] -> Mitigation: clear actionable error messages, retry guidance, and explicit offline behavior.
- [Embedding storage/index time growth] -> Mitigation: batched generation, incremental updates, and compact vector serialization.
- [Cross-platform model/runtime issues] -> Mitigation: startup checks, deterministic fallback to lexical mode when neural mode cannot initialize.
- [Ranking disagreements between lexical and semantic outputs] -> Mitigation: explicit retrieval mode and stable output schema.

## Migration Plan

1. Add model cache directory and bootstrap/check logic under `.flashgrep/`.
2. Extend DB/index schema to store chunk vectors and model metadata.
3. Generate vectors in indexing and update/remove vectors in incremental paths.
4. Add CLI query retrieval mode and MCP query schema support.
5. Add tests/docs for bootstrap, retrieval behavior, and deterministic outputs.

Rollback strategy:
- Disable neural retrieval mode while preserving lexical index/query behavior.
- Keep lexical artifacts untouched and compatible with previous releases.

## Open Questions

- Should we provide an explicit `model pull` command for CI/air-gapped workflows?
- What default embedding batch size best balances speed and memory for low-end machines?
- Do we require checksum pinning for all downloaded model assets beyond manifest validation?
