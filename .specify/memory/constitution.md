<!--
Sync Impact Report
- Version change: template -> 1.0.0
- Modified principles:
  - PRINCIPLE_1_NAME -> I. Index-First Speed
  - PRINCIPLE_2_NAME -> II. Deterministic and Predictable Behavior
  - PRINCIPLE_3_NAME -> III. CLI-First Explicitness
  - PRINCIPLE_4_NAME -> IV. Safety and Reliability by Default
  - PRINCIPLE_5_NAME -> V. Flashgrep-First Operations
- Added sections:
  - Engineering Constraints
  - Development Workflow and Quality Gates
- Removed sections:
  - None
- Templates requiring updates:
  - ✅ updated: .specify/templates/plan-template.md
  - ✅ updated: .specify/templates/spec-template.md
  - ✅ updated: .specify/templates/tasks-template.md
  - ✅ reviewed (no files present): .specify/templates/commands/*.md
- Follow-up TODOs:
  - None
-->

# Flashgrep Constitution

## Core Principles

### I. Index-First Speed
Flashgrep MUST prioritize index-backed operations over repeated filesystem scans.
New features MUST preserve fast repeated query behavior on large repositories and
MUST document measurable performance expectations (latency and memory) for changed
paths. Any change that materially regresses query speed or memory overhead without
an approved mitigation plan is non-compliant.

Rationale: Flashgrep exists to be a fast, low-overhead replacement for repetitive
grep/glob workflows at scale.

### II. Deterministic and Predictable Behavior
Given identical inputs, configuration, and index state, Flashgrep MUST return
deterministic output ordering and stable error semantics. Features that introduce
heuristics MUST define explicit fallback rules and typed failure reasons. Output
formats MUST remain script-friendly and backward compatible unless a versioned
breaking change is approved.

Rationale: Power users and automation rely on repeatable behavior, not hidden
variability.

### III. CLI-First Explicitness
Every externally visible capability MUST be accessible through the CLI and MUST use
clear, explicit options rather than implicit behavior. Pattern modes, include/exclude
scope, and retrieval mode selection MUST be user-directed and documented. Hidden
magic, ambiguous defaults, and side effects without user intent are prohibited.

Rationale: Flashgrep is built to feel simple like fgrep while remaining powerful
for advanced workflows.

### IV. Safety and Reliability by Default
Operations MUST fail safely with actionable diagnostics and MUST avoid destructive
behavior unless explicitly requested by flags or parameters. File and index handling
MUST protect data integrity, and cross-platform path behavior MUST stay consistent.
Typed errors and bounded resource usage are mandatory for automation-facing APIs.

Rationale: Safe, reliable behavior is required for unattended tooling and CI usage.

### V. Flashgrep-First Operations
Project guidance, automation flows, and agent-facing docs MUST prefer Flashgrep
native capabilities (`query`, `glob`/`files`, `read_code`, `write_code`, `fs_*`).
Fallback to host-native grep/glob/read/write tools is allowed ONLY when Flashgrep
cannot satisfy the operation, and the fallback reason MUST be explicit.

Rationale: Flashgrep is intended as the primary search and file-ops layer, with
native tools reserved for constrained fallback cases.

## Engineering Constraints

- Implementation language for core components MUST remain Rust.
- Core search and filesystem pathways MUST minimize overhead and avoid unnecessary
  allocations in hot paths.
- Pattern handling MUST be explicit (`smart`, `literal`, `regex`) and consistent
  across CLI and MCP surfaces.
- Features MUST scale to large codebases and include limits/pagination where output
  can grow unbounded.
- Public behavior changes MUST be reflected in README and relevant docs before merge.

## Development Workflow and Quality Gates

- Work MUST be spec-driven: each non-trivial change starts with spec/plan/tasks
  artifacts and a Constitution Check.
- Changes to search semantics, sorting, matching, routing, or file operations MUST
  include tests that cover deterministic behavior and failure paths.
- Performance-sensitive changes MUST include benchmark or regression evidence for
  affected commands/APIs.
- Review and release gates MUST verify CLI and MCP parity for changed behaviors.
- Docs and examples MUST remain aligned with shipped behavior before release.

## Governance

This constitution is the highest-priority engineering policy for Flashgrep.
All plans, specs, tasks, and reviews MUST include a compliance check against these
principles.

Amendment process:
- Propose changes through a documented spec update that includes rationale,
  migration impact, and affected templates/docs.
- Obtain maintainer approval before merging governance changes.
- Update dependent templates and runtime guidance in the same change set.

Versioning policy (semantic versioning for governance text):
- MAJOR: Removes or redefines principles in a backward-incompatible way.
- MINOR: Adds a principle/section or materially expands mandatory guidance.
- PATCH: Clarifies language, fixes typos, or improves wording without changing
  normative requirements.

Compliance review expectations:
- Every feature plan MUST pass Constitution Check gates before implementation.
- Pull requests MUST list constitution-relevant impacts and verification evidence.
- Release readiness reviews MUST confirm docs, templates, and behavior remain
  consistent with this constitution.

**Version**: 1.0.0 | **Ratified**: 2026-03-06 | **Last Amended**: 2026-03-06
