# Implementation Plan: Enforce Native Policy Routing

**Branch**: `001-enforce-native-policy` | **Date**: 2026-03-06 | **Spec**: `specs/001-enforce-native-policy/spec.md`
**Input**: Feature specification from `/specs/001-enforce-native-policy/spec.md`

## Summary

Strengthen policy enforcement so agent sessions are native-tool first by default,
reject ungated fallback routes with typed diagnostics, support deterministic drift
recovery, and keep docs/skills parity gated before release.

## Technical Context

**Language/Version**: Rust edition 2021  
**Primary Dependencies**: `serde_json`, `sha2`, `tokio`, `clap`, existing MCP stack (`mcp-server`, `mcp-protocol`)  
**Storage**: Runtime session metadata payloads, file-based docs/skills artifacts, existing local metadata DB for operational checks  
**Testing**: `cargo test`, `cargo clippy --all-targets -- -D warnings`, policy parity tests for docs/skills/runtime fields  
**Target Platform**: Cross-platform local CLI + MCP server (Windows/macOS/Linux)  
**Project Type**: Rust CLI and MCP service  
**Performance Goals**: Enforcement checks add no more than 10 ms p95 overhead to tool routing decisions; drift detection occurs within one request cycle; no measurable regression to indexed query latency  
**Constraints**: Deterministic route decisions; typed policy violations; explicit fallback gate semantics; backward-compatible bootstrap fields; fail-safe behavior on policy mismatch  
**Scale/Scope**: All MCP sessions and bootstrap aliases; native routing families for discovery/read/write flows; release-time parity checks across runtime policy + docs + skills

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Pre-Research Gate

- [x] **Index-First Speed**: Plan keeps enforcement in routing metadata and avoids scan-path changes.
- [x] **Determinism**: Policy outcomes are typed and deterministic for identical inputs/session state.
- [x] **CLI-First Explicitness**: Strict/fallback behavior is explicit via policy metadata and gate reason codes.
- [x] **Safety & Reliability**: Default behavior is deny-by-default for ungated fallback routes with actionable diagnostics.
- [x] **Flashgrep-First Routing**: Native tools remain primary and mandatory unless explicit gate criteria pass.
- [x] **Docs & Parity**: Plan includes docs/skills parity checks as release gates.

Pre-research result: PASS.

### Post-Design Gate

- [x] **Index-First Speed**: Design confines changes to policy routing, bootstrap metadata, and validation tests.
- [x] **Determinism**: Contracts define stable decision states (`allowed_native`, `allowed_fallback`, `denied`) and typed reason codes.
- [x] **CLI-First Explicitness**: Fallback gates and strict enforcement are represented as explicit contract fields and docs.
- [x] **Safety & Reliability**: Drift detection and force reinjection are deterministic with fail-safe behavior.
- [x] **Flashgrep-First Routing**: Skills/docs/runtime contract all declare Flashgrep-native first routing order.
- [x] **Docs & Parity**: Quickstart and contract include parity validation for README + skills + runtime payload fields.

Post-design result: PASS.

## Project Structure

### Documentation (this feature)

```text
specs/001-enforce-native-policy/
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── policy-enforcement-contract.md
└── tasks.md
```

### Source Code (repository root)

```text
src/
├── mcp/
│   ├── bootstrap.rs
│   ├── skill.rs
│   ├── stdio.rs
│   ├── mod.rs
│   └── safety.rs
├── cli/
└── search/

skills/
└── SKILL.md

docs/
└── *.md

tests/
├── integration_tests.rs
└── mcp_integration_tests.rs
```

**Structure Decision**: Keep a single Rust project and implement policy enforcement
in MCP routing/bootstrap layers, with synchronized docs/skills and integration tests.

## Complexity Tracking

No constitution violations identified; no exceptions required.
