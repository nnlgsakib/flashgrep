## 1. Shared Continuation Contract

- [x] 1.1 Define shared continuation metadata types (cursor, chunk index, completed flag, exact boundaries).
- [x] 1.2 Implement reusable continuation planners/helpers for read, write, query, and glob operations.
- [x] 1.3 Add compatibility mapping so small operations still return legacy-compatible one-shot responses.

## 2. Unbounded Read/Write Implementation

- [x] 2.1 Refactor `read_code` to support automatic multi-part retrieval for arbitrarily large logical requests.
- [x] 2.2 Refactor `get_slice` to support continuation-based large-range retrieval without precision loss.
- [x] 2.3 Implement chunked large-write workflow in `write_code` with exact range/ordering guarantees and resumable progress metadata.
- [x] 2.4 Preserve precondition safety semantics across multi-part write flows.

## 3. MCP Transport Stability for Large Operations

- [x] 3.1 Update stdio handler to process continuation workflows without session drops across many chunks.
- [x] 3.2 Update TCP handler to mirror stdio continuation and error semantics for large operations.
- [x] 3.3 Ensure structured recoverable errors for malformed continuation state while keeping transport sessions active.

## 4. Grep/Glob Large-Scale Completion

- [x] 4.1 Extend query execution to return deterministic continuation windows for very large result sets.
- [x] 4.2 Extend glob traversal/output to provide deterministic no-gap/no-duplicate continuation over very large repositories.
- [x] 4.3 Validate CLI parity for continuation-based query/files flows with stable ordering guarantees.

## 5. Verification and Documentation

- [x] 5.1 Add regression tests for unbounded logical reads/writes across many chunks with exact reconstruction checks.
- [x] 5.2 Add regression tests for large query/glob continuation ordering (no duplicates, no gaps).
- [x] 5.3 Add cross-transport MCP tests proving session continuity through repeated large continuation sequences.
- [x] 5.4 Update README and `skills/SKILL.md` with continuation-loop usage guidance for large operations.
