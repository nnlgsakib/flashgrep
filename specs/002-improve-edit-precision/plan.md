# Implementation Plan: Precise Editing and Patch Reliability

**Branch**: `002-improve-edit-precision` | **Date**: 2026-03-06 | **Spec**: `specs/002-improve-edit-precision/spec.md`
**Input**: Feature specification from `/specs/002-improve-edit-precision/spec.md`

## Summary

Improve edit precision so line-range writes cannot silently duplicate or overwrite
unexpected content, add explicit batch editing behavior with deterministic outcomes,
and synchronize docs/skills so automation follows the new safety model.

## Technical Context

**Language/Version**: Rust edition 2021  
**Primary Dependencies**: `serde_json`, `sha2`, `tokio`, `clap`, existing MCP stack (`mcp-server`, `mcp-protocol`)  
**Storage**: Local files for edits, SQLite metadata (`.flashgrep/metadata.db`), temporary continuation session files  
**Testing**: `cargo test` (unit + integration-style tests in MCP/code I/O modules), CLI/MCP contract checks  
**Target Platform**: Cross-platform CLI and MCP server (Windows/macOS/Linux)  
**Project Type**: Rust CLI + local MCP server  
**Performance Goals**: Single precise write keeps p95 latency under 150 ms for files <= 2 MB; batch write of 100 operations completes under 2 s on a warm local workspace; no regression in indexed query speed  
**Constraints**: Deterministic results for identical inputs; typed failure modes; explicit user-controlled batch mode; bounded payload sizes; backward-compatible defaults  
**Scale/Scope**: Up to 500 operations per batch request; mixed file and range targets across large repositories

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gate

- [x] **Index-First Speed**: Edits remain minimal-range and avoid broad rewrites.
- [x] **Determinism**: Plan defines deterministic order and per-operation typed outcomes.
- [x] **CLI-First Explicitness**: Batch mode and conflict handling are explicit inputs.
- [x] **Safety & Reliability**: Preconditions and conflict rejection are core behavior.
- [x] **Flashgrep-First Routing**: Uses native `write_code` and `batch_write_code` paths.
- [x] **Docs & Parity**: Includes docs and skill updates in required deliverables.

Pre-research result: PASS.

### Post-Design Gate

- [x] **Index-First Speed**: Design limits rewrite scope to explicit line ranges and
      preserves current read/write budget limits.
- [x] **Determinism**: Batch contract specifies ordered execution and stable status
      schema (`applied|failed|conflict|skipped`) with typed reason codes.
- [x] **CLI-First Explicitness**: Contracts define explicit `mode` (`atomic` or
      `best_effort`), preconditions, and dry-run preview behavior.
- [x] **Safety & Reliability**: Preflight overlap checks and precondition validation
      block stale or ambiguous writes.
- [x] **Flashgrep-First Routing**: Quickstart and contracts only use Flashgrep
      native tools for edit flows.
- [x] **Docs & Parity**: Design includes README + `skills/SKILL.md` alignment and
      validation in release checks.

Post-design result: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/002-improve-edit-precision/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── editing-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── mcp/
│   ├── code_io.rs
│   ├── tools.rs
│   ├── stdio.rs
│   └── safety.rs
├── cli/
│   ├── mod.rs
│   └── fs.rs
└── search/
    └── mod.rs

skills/
└── SKILL.md

docs/
└── *.md

tests/ (existing Rust test modules under src/ via `cargo test`)
```

**Structure Decision**: Keep single Rust project layout; implement behavior in MCP
editing modules and keep guidance synchronized in `skills/` and `docs/`.

## Complexity Tracking

No constitution violations identified; no exception tracking required.
