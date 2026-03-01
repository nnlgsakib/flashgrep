## Why

Flashgrep currently excels at deterministic programmatic search, but users also need intent-based natural-language discovery (for example, asking where auth logic lives). Adding a local neural command path now improves developer productivity while preserving Flashgrep's CLI-first, local-only design.

## What Changes

- Add a fully functional neural command workflow for natural-language project queries.
- Add first-run model bootstrap that downloads and caches `BAAI/bge-small-en-v1.5` in `.flashgrep/model-cache/`.
- Extend indexing to generate and persist embeddings from project chunks for semantic retrieval.
- Add CLI neural interface controls so users can run lexical, semantic, or hybrid queries explicitly.
- Return deterministic, path-first results (file path, line range, score, preview) for neural operations.

## Capabilities

### New Capabilities
- `neural-command-support`: Embedded model lifecycle, vector search, and natural-language query handling over project-local vectors.

### Modified Capabilities
- `cli-search-commands`: Add explicit neural CLI query mode and stable output fields for semantic/hybrid results.
- `indexing-engine`: Add embedding generation/storage during indexing and incremental updates.
- `search-engine`: Add semantic retrieval and deterministic hybrid ranking behavior.

## Impact

- Affected code: CLI query parsing/output, index pipeline, search engine, metadata storage, model bootstrap runtime.
- Data layout: `.flashgrep/` gains model cache files and vector artifacts.
- Dependencies: Embedded model bootstrap/inference support and vector utilities.
- Runtime behavior: First neural use may perform one-time model download; subsequent runs reuse local cache.
