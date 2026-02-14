## 1. API Contracts and Validation

- [x] 1.1 Define request/response schemas for token-efficient read operations (`file_path`/slice mode, `symbol_name` mode, budget fields, metadata levels).
- [x] 1.2 Define request/response schemas for minimal-diff write operations (line-range replacement, optional preconditions, structured conflict payload).
- [x] 1.3 Implement input validation and normalization for limits (`max_tokens`, `max_bytes`, `max_lines`) and mutually exclusive read mode fields.

## 2. Read Operation Implementation

- [x] 2.1 Implement deterministic budget enforcement across token/byte/line caps with clear precedence and truncation markers.
- [x] 2.2 Implement continuation cursor generation/consumption so truncated reads can resume without overlap.
- [x] 2.3 Implement symbol-scoped reads using existing symbol/index facilities with configurable context lines.
- [x] 2.4 Implement metadata verbosity profiles (`minimal`, `standard`) and ensure minimal mode suppresses non-essential fields.

## 3. Write Operation Implementation

- [x] 3.1 Implement line-range replacement engine that applies only the requested slice and preserves surrounding file content.
- [x] 3.2 Implement optimistic concurrency precondition checks (file hash and boundary text) before mutation.
- [x] 3.3 Return structured conflict responses with actionable mismatch details for automated retry/rebase.

## 4. Integration and Tool Surface

- [x] 4.1 Register new MCP tools/endpoints for optimized read and write operations.
- [x] 4.2 Wire tool handlers to existing repository/index infrastructure and ensure consistent path/symbol resolution behavior.
- [x] 4.3 Add concise user-facing/tooling docs for parameters, defaults, continuation flow, and conflict handling.

## 5. Verification and Performance

- [x] 5.1 Add unit tests for budget enforcement, continuation correctness, metadata profiles, and validation errors.
- [x] 5.2 Add unit/integration tests for write success paths and precondition conflict handling.
- [x] 5.3 Add benchmarks or measurement tests demonstrating reduced token/byte output versus current baseline flows.
- [x] 5.4 Run full test suite and confirm no regressions in existing search/index functionality.
