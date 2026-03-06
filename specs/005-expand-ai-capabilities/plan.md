# Implementation Plan: AI Capability Expansion and Efficiency

**Branch**: `005-expand-ai-capabilities` | **Date**: 2026-03-06 | **Spec**: `specs/005-expand-ai-capabilities/spec.md`
**Input**: Feature specification from `/specs/005-expand-ai-capabilities/spec.md`

## Summary

Expand Flashgrep AI usefulness in high-value workflows (AI-first discovery,
token-aware context assembly, and governed prompt behavior), while preserving
deterministic routing, typed fallbacks, and strict Flashgrep-native policy.

## Technical Context

**Language/Version**: Rust edition 2021  
**Primary Dependencies**: existing `async-openai`, `serde_json`, `tokio`, `clap`, MCP stack (`mcp-server`, `mcp-protocol`)  
**Storage**: local metadata DB (`.flashgrep/metadata.db`), runtime session metadata payloads, docs/skills artifacts  
**Testing**: `cargo test`, `cargo clippy --all-targets -- -D warnings`, integration parity checks for runtime/docs/skills  
**Target Platform**: cross-platform local CLI + MCP server (Windows/macOS/Linux)  
**Project Type**: Rust CLI + MCP service  
**Performance Goals**: AI routing adds <= 15 ms p95 control-plane overhead; token packing enforces configured budget in >= 95% requests; deterministic fallback response produced in <= 1 request cycle on AI unavailability  
**Constraints**: deterministic outputs for equal inputs, explicit AI modes, typed denials, bounded token budgets, no regression in index-backed query behavior  
**Scale/Scope**: repository-scale workflows (up to millions of LOC), AI scope metadata and policy checks on all AI-assisted MCP flows

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gate

- [x] **Index-First Speed**: Plan keeps index-backed retrieval primary; AI applies after deterministic candidate selection or explicit mode switch.
- [x] **Determinism**: Route decisions and fallback reasons are typed and stable for identical state.
- [x] **CLI-First Explicitness**: AI behavior uses explicit modes/flags and published metadata fields.
- [x] **Safety & Reliability**: Prompt-policy checks and fail-safe fallbacks are mandatory.
- [x] **Flashgrep-First Routing**: Native Flashgrep tools remain primary path; fallback is gated and auditable.
- [x] **Docs & Parity**: Plan includes README/skills/docs parity checks before release.

Pre-research result: PASS.

### Post-Design Gate

- [x] **Index-First Speed**: Design preserves deterministic retrieval path and adds bounded AI post-processing stages.
- [x] **Determinism**: Contract defines typed route states and denial/fallback reason codes.
- [x] **CLI-First Explicitness**: Contract requires explicit AI mode, budget profile, and policy metadata fields.
- [x] **Safety & Reliability**: Policy engine checks execute before AI actions; force recovery path is explicit.
- [x] **Flashgrep-First Routing**: AI scope is constrained to approved flows; host-native fallback remains gated.
- [x] **Docs & Parity**: Quickstart includes runtime/docs/skills parity validation steps.

Post-design result: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/005-expand-ai-capabilities/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── ai-governance-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── mcp/
│   ├── bootstrap.rs
│   ├── mod.rs
│   ├── stdio.rs
│   ├── skill.rs
│   └── safety.rs
├── search/
│   └── mod.rs
└── cli/

skills/
└── SKILL.md

docs/
└── *.md

tests/
├── integration_tests.rs
└── mcp_integration_tests.rs
```

**Structure Decision**: Keep single Rust project architecture; implement AI scope,
prompt-budget governance, and route-policy checks in MCP/search layers with matching
docs/skills parity tests.

## Complexity Tracking

No constitution violations identified; no exception tracking required.
