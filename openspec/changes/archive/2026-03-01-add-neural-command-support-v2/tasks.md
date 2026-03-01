## 1. Model Bootstrap and Runtime Setup

- [x] 1.1 Add/verify embedded neural dependencies and feature-gated build wiring for local model support.
- [x] 1.2 Implement first-run model bootstrap for `BAAI/bge-small-en-v1.5` into `.flashgrep/model-cache/`.
- [x] 1.3 Add cache validation, deterministic error messages, and offline handling guidance for bootstrap failures.

## 2. Vector Indexing Pipeline

- [x] 2.1 Extend indexing metadata schema/storage to persist chunk vectors with model id, hash, and location coordinates.
- [x] 2.2 Generate embeddings during initial indexing using existing chunk boundaries.
- [x] 2.3 Implement incremental vector update/removal for changed, deleted, and ignored files.

## 3. Search and Ranking Integration

- [x] 3.1 Add semantic retrieval mode that queries project-local vectors and returns ranked chunk matches.
- [x] 3.2 Add deterministic hybrid ranking that combines lexical and semantic signals.
- [x] 3.3 Ensure lexical, semantic, and hybrid outputs expose stable path/line/score fields in text and JSON modes.

## 4. CLI and Validation

- [x] 4.1 Add CLI neural interface controls for explicit retrieval mode selection (`lexical`, `semantic`, `hybrid`).
- [x] 4.2 Wire first neural query execution to trigger bootstrap when model cache is missing.
- [x] 4.3 Add tests for bootstrap behavior, cached reuse, semantic/hybrid ranking behavior, and deterministic ordering.
- [x] 4.4 Update user docs for neural query usage, model cache location, and troubleshooting.
