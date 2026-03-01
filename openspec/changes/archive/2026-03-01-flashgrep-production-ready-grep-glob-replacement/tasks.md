## 1. Compatibility Contract Baseline

- [x] 1.1 Build a grep/glob parity matrix covering current behavior vs required behavior from updated specs
- [x] 1.2 Define canonical exit-status, stdout/stderr, and error-model contracts for script-safe execution
- [x] 1.3 Add conformance fixture sets for grep-like and glob-like workflows used in CI automation

## 2. Grep Compatibility Implementation

- [x] 2.1 Implement fixed-string multi-pattern matching and wire it to query command options
- [x] 2.2 Implement deterministic grep-compatible exit code handling for match, no-match, and failure outcomes
- [x] 2.3 Implement machine-oriented output mode with stable fields for file, line, match, and context metadata
- [x] 2.4 Normalize include/exclude path scope handling across platform-native separators

## 3. Glob Parity and Determinism

- [x] 3.1 Extend glob engine for recursive wildcard and character-class parity under documented semantics
- [x] 3.2 Implement explicit hidden and dot-path inclusion policy controls
- [x] 3.3 Enforce deterministic ordering and no-gap/no-duplicate continuation behavior across platforms
- [x] 3.4 Add cross-platform integration tests for depth bounds, filters, sorting, and continuation windows

## 4. Filesystem Operations Capability

- [x] 4.1 Add CLI filesystem command group scaffolding for create, list, stat, copy, move, and remove
- [x] 4.2 Implement file and directory lifecycle operations with deterministic error mapping
- [x] 4.3 Implement copy/move overwrite semantics and conflict handling
- [x] 4.4 Implement dry-run and force safety controls for destructive and overwrite operations
- [x] 4.5 Implement structured machine-readable output for list/stat operations with stable field names

## 5. CLI UX and Cross-Cutting Integration

- [x] 5.1 Integrate grep-compat and glob-replacement options into existing query/files command surfaces
- [x] 5.2 Ensure deterministic output formatting and bounded response behavior in all new command paths
- [x] 5.3 Add platform abstraction integration for shared path normalization across search, glob, and filesystem commands

## 6. Documentation, Skill, and Release Readiness

- [x] 6.1 Update `README.md` with grep/glob migration mappings, filesystem command usage, and production guardrails
- [x] 6.2 Update `skills/SKILL.md` with Flashgrep-first routing guidance including filesystem operation workflows
- [x] 6.3 Add release-readiness checklist for cross-platform conformance, performance baselines, and rollback controls
- [ ] 6.4 Run full test matrix (unit, integration, cross-platform parity) and resolve blocking regressions
