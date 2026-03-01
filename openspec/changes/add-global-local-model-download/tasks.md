## 1. Configuration and Path Resolution

- [x] 1.1 Extend `.flashgrep/config.json` schema with a global model cache path setting and defaults
- [x] 1.2 Add config validation and normalization for the global model cache path
- [x] 1.3 Introduce model storage scope types and a shared cache path resolver used by cache checks and downloads

## 2. Download Prompt and Runtime Behavior

- [x] 2.1 Update interactive startup flow to ask for download scope (`global` or `local`) after download consent
- [x] 2.2 Implement non-interactive deterministic scope behavior without prompting
- [x] 2.3 Update download/cache messaging to show resolved destination and guidance for missing global config

## 3. Neural Cache Integration

- [x] 3.1 Refactor model manifest and cache helper functions to accept resolved cache roots
- [x] 3.2 Ensure existing global model manifests are detected and skip redundant downloads
- [x] 3.3 Keep local cache behavior fully functional and backward compatible

## 4. Verification and Documentation

- [x] 4.1 Add or update unit tests for scope selection, config path handling, and cached-model reuse
- [x] 4.2 Add or update integration tests for interactive and non-interactive startup paths
- [x] 4.3 Update user-facing docs/examples for global vs local model download setup
