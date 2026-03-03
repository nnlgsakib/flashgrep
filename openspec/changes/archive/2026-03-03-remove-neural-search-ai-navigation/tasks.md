## 1. Remove neural runtime and indexing surfaces

- [x] 1.1 Remove neural module usage and model prompt entry points from CLI and watcher startup paths.
- [x] 1.2 Remove semantic/vector retrieval code paths and data access calls from search execution flow.
- [x] 1.3 Remove neural-related dependencies, feature flags, and unused model-cache helpers from the build.

## 2. Update CLI and MCP contracts to lexical-only retrieval

- [x] 2.1 Update query argument validation and retrieval-mode parsing to reject removed `semantic` and `hybrid` modes deterministically.
- [x] 2.2 Update MCP tool schemas/metadata and bootstrap policy payloads to remove neural-first routing semantics.
- [x] 2.3 Keep existing lexical query behavior and output structure unchanged for non-AI workflows.

## 3. Align docs and policy guidance

- [x] 3.1 Update `README.md` and skill/bootstrap documentation to remove neural model setup and semantic/hybrid usage examples.
- [x] 3.2 Update embedded policy guidance and AI-agent docs to describe programmatic-first routing and existing fallback gates.

## 4. Preserve non-AI behavior with regression coverage

- [x] 4.1 Update and/or remove tests tied to neural prompting, semantic retrieval, and neural routing metadata.
- [x] 4.2 Add regression assertions for lexical query behavior, index-backed file search, and MCP filesystem/symbol tool stability.
- [x] 4.3 Run project test suite slices relevant to CLI, search, watcher, and MCP to confirm non-AI features remain intact.
