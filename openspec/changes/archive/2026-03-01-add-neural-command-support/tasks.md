## 1. Model Bootstrap and Runtime Setup

- [x] 1.1 Add embedded inference/embedding dependencies and wire feature-gated build configuration for local model execution.
- [x] 1.2 Implement `.flashgrep/model-cache/` layout and first-run download flow for `BAAI/bge-small-en-v1.5` with cache validation.
- [x] 1.3 Add deterministic error handling and user guidance for download failure, offline mode, and corrupted cache.

## 2. Vector Indexing Pipeline

- [x] 2.1 Extend indexing data model/storage to persist embedding metadata and vector references for each indexed chunk.
- [x] 2.2 Implement embedding generation during initial indexing using existing chunk boundaries.
- [x] 2.3 Implement ignore-aware incremental embedding updates for added/changed/removed files.

## 3. Neural Query and Ranking

- [x] 3.1 Add semantic retrieval execution path that queries project vectors and returns ranked chunk matches.
- [x] 3.2 Implement deterministic hybrid ranking that combines vector similarity with existing metadata signals.
- [x] 3.3 Ensure query responses include stable path/line/score fields for text and JSON output modes.

## 4. CLI Integration and Validation

- [x] 4.1 Add explicit neural mode support to query CLI commands without changing default lexical behavior.
- [x] 4.2 Implement first-use neural command flow that triggers model bootstrap when cache is missing.
- [x] 4.3 Add tests covering first-run download, cached reuse, semantic query results, and deterministic ordering.
- [x] 4.4 Add/update docs for neural mode usage, model cache location, and troubleshooting.
