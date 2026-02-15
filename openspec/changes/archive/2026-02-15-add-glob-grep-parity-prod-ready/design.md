## Context

Flashgrep already provides index-backed query and glob features, but production teams still rely on classic `grep` and filesystem glob tooling for options not fully exposed through CLI and MCP contracts. This change spans search execution, file discovery, CLI contracts, MCP schemas, and user/agent documentation. The core constraint is preserving Flashgrep's deterministic and fast index-first behavior while expanding option coverage enough to serve as a practical drop-in replacement for repeated grep/glob workflows.

## Goals / Non-Goals

**Goals:**
- Provide complete, production-ready grep-parity behavior for indexed content search with stable, automation-safe output.
- Provide complete, production-ready glob-parity behavior with explicit filtering/traversal controls and deterministic sorting/limits.
- Keep parity options consistent between CLI and MCP so scripts and agents can switch surfaces without semantic drift.
- Update `README.md` and `skills/SKILL.md` with concise migration and usage guidance.

**Non-Goals:**
- Re-implement shell-level `grep`/`glob` byte-for-byte or emulate every obscure flag across all platforms.
- Replace one-off ad hoc shell usage for tiny folders where indexing is not desired.
- Introduce semantic/code-intelligence searching beyond text/metadata parity scope in this change.

## Decisions

### Decision: Treat parity as contract-level compatibility, not CLI flag cloning
Implement parity via a documented behavior contract (regex, case options, path scoping, context controls, deterministic ordering/limits) instead of strict 1:1 command-line flag mirroring.

- Rationale: preserves cross-platform consistency and allows index-first optimizations.
- Alternative considered: exact GNU/BSD flag parity. Rejected due to platform variance and fragile compatibility guarantees.

### Decision: Unify option semantics through shared internal option models
Define shared option models used by CLI and MCP layers for query/glob to avoid drift in validation, defaults, and output behavior.

- Rationale: one source of truth keeps behavior stable and reduces regression risk.
- Alternative considered: independent parsers per surface. Rejected because it duplicates logic and creates divergence risk.

### Decision: Preserve deterministic bounded output as a hard requirement
All new parity options must honor explicit caps and deterministic ordering so automation remains reliable across runs.

- Rationale: predictable behavior is critical for CI, agent loops, and large-repo workloads.
- Alternative considered: best-effort streaming without strict ordering. Rejected due to nondeterministic results.

### Decision: Optimize skills/docs for token efficiency using compact patterns
Revise `skills/SKILL.md` to focus on compact decision trees, default tool order, and minimal examples; move detailed parameter matrices to README/API docs.

- Rationale: reduces agent token overhead while preserving correct operational guidance.
- Alternative considered: keep verbose all-in-one skill docs. Rejected due to repeated context bloat.

## Risks / Trade-offs

- [Parity scope creep] -> Define required parity behaviors in specs and defer edge-case flags to future deltas.
- [Performance regressions from broader filtering/options] -> Keep traversal/query pruning early, benchmark key workloads, and enforce limit-first execution.
- [CLI/MCP contract drift] -> Use shared models and add compatibility tests covering both interfaces.
- [Documentation mismatch with implementation] -> Treat README and skill updates as required tasks in apply-ready artifacts.

## Migration Plan

1. Add/align internal option models for query and glob contracts.
2. Extend engines and validators to support parity features while preserving defaults.
3. Update CLI and MCP wiring to expose shared semantics.
4. Add/expand automated tests for parity, determinism, and bounded output behavior.
5. Update `README.md` and `skills/SKILL.md` with migration and low-token usage guidance.
6. Roll out behind stable defaults so existing calls continue working.

Rollback strategy: if regressions appear, disable newly exposed parity options while preserving existing baseline query/glob behavior and docs notes.

## Open Questions

- Which regex engine constraints (if any) should be explicitly documented as non-goals for strict grep parity?
- Should compatibility reporting include a machine-readable "supported parity matrix" for CLI and MCP consumers?
