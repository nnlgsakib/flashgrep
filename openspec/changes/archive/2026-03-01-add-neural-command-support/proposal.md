## Why

Flashgrep currently focuses on programmatic lexical operations, which makes intent-based discovery harder for users who want to ask higher-level questions (for example, where authentication logic lives). Adding an embedded neural retrieval path now enables semantic workflows without introducing external services.

## What Changes

- Add an embedded neural command workflow that supports natural-language project queries and returns relevant file paths and contextual snippets.
- Add first-run model bootstrap behavior that downloads and caches `BAAI/bge-small-en-v1.5` under `.flashgrep/`.
- Add vector indexing support so project chunks can be embedded and queried semantically using repository-local data.
- Add CLI-facing controls for neural retrieval mode, including deterministic output formatting and graceful fallback when model assets are unavailable.

## Capabilities

### New Capabilities
- `neural-command-support`: Embedded model lifecycle, vector retrieval, and natural-language query execution over indexed project content.

### Modified Capabilities
- `cli-search-commands`: Extend CLI search flows with a neural operation mode and consistent path-oriented result output.
- `indexing-engine`: Extend indexing to generate/store embeddings from project chunks for semantic retrieval.
- `search-engine`: Add semantic retrieval and ranking behavior that can surface intent-matched code locations.

## Impact

- Affected code: CLI command parsing and output formatting, indexing pipeline, search execution path, and storage/runtime model loading.
- Data/layout: `.flashgrep/` gains model cache assets and vector index/metadata artifacts.
- Dependencies: Adds embedded inference and embedding pipeline dependencies needed to run `BAAI/bge-small-en-v1.5` locally.
- Runtime behavior: First model use may incur a one-time download/setup step; subsequent runs use cached local assets.
