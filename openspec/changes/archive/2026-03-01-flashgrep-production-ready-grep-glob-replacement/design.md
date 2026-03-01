## Context

Flashgrep already provides indexed search and glob-like discovery, but users still need to fall back to legacy `grep`, shell globbing, and direct filesystem commands for common automation workflows. The current gap is not raw speed; it is behavioral coverage and operational completeness across Linux, macOS, and Windows. This change must preserve Flashgrep's deterministic, bounded-output model while adding compatibility semantics and filesystem command support suitable for production scripting.

## Goals / Non-Goals

**Goals:**
- Deliver grep-replacement behavior for high-value script paths: matching modes, exit-status contracts, stable output formatting, and predictable error handling.
- Close glob parity gaps for recursive matching, hidden-path handling, include/exclude composition, and cross-platform path behavior.
- Add cross-platform filesystem operations (create/list/stat/copy/move/remove for files and directories) with explicit safety and error contracts.
- Keep all new operations deterministic, automation-friendly, and index-aware where appropriate.
- Update skills and README guidance so users and agents can safely adopt Flashgrep as the default tool.

**Non-Goals:**
- Full byte-for-byte compatibility with every historical or platform-specific `grep` implementation quirk.
- Replacing shell semantics unrelated to search and file operations (job control, pipes, shell expansion precedence).
- Implementing privileged or ACL-management operations outside standard user-level filesystem interactions.

## Decisions

### Decision: Compatibility-first command contracts
Flashgrep will define explicit compatibility profiles for grep and glob workflows rather than loosely approximating behavior. Priority is given to script-critical contracts (exit codes, stdout/stderr boundaries, stable ordering, and flag behavior), with deterministic behavior documented when exact legacy parity would conflict with safety or predictability.

Alternatives considered:
- Full strict emulation mode for each platform-specific grep variant: rejected as high complexity and brittle.
- Keep current best-effort behavior without formal contracts: rejected because it blocks production replacement confidence.

### Decision: Unified cross-platform path and filesystem abstraction
Filesystem operations and glob/search path handling will run through a shared path-normalization and IO abstraction layer. This centralizes separator normalization, case-handling policy, symlink traversal rules, and consistent error mapping across platforms.

Alternatives considered:
- Keep per-command ad hoc path logic: rejected due to inconsistency risk.
- Hard-normalize all paths to POSIX display only: rejected because native-path fidelity is needed for local scripting.

### Decision: Safety gates for mutating filesystem operations
Mutating operations (remove/move/overwrite) will require explicit confirmation flags or no-prompt force flags with strict deterministic behavior. Dry-run support and structured error reporting are required to make operations automation-safe.

Alternatives considered:
- Always allow mutation without guardrails: rejected for production safety reasons.
- Interactive confirmations only: rejected because automation and CI require non-interactive controls.

### Decision: Spec/test-driven production readiness
Each compatibility and filesystem requirement must map to integration-level acceptance scenarios that run on all supported platforms. Regression baselines for grep/glob parity and filesystem behavior will gate release readiness.

Alternatives considered:
- Unit-test-only validation: rejected because behavior parity must be validated end-to-end.
- Platform-specific manual verification: rejected due to repeatability and release risk.

## Risks / Trade-offs

- [Parity surface may expand beyond initial scope] -> Mitigation: prioritize top script-critical workflows, then phase lower-value edge cases.
- [Cross-platform filesystem differences can cause subtle divergence] -> Mitigation: enforce shared abstraction plus platform-matrix integration tests.
- [Safety gates may feel stricter than legacy tools] -> Mitigation: provide explicit opt-in force flags and clear migration docs.
- [Added capability surface may impact maintainability] -> Mitigation: modular command architecture, consistent error model, and conformance tests per capability.

## Migration Plan

1. Land spec updates and add conformance test harnesses for grep, glob, and filesystem operations.
2. Implement compatibility contracts behind stable CLI flags and documented defaults.
3. Add filesystem operation commands with dry-run and force semantics.
4. Update skill and README migration guidance with command mappings and production guardrails.
5. Run cross-platform test matrix and compatibility baselines before release tag.

Rollback strategy:
- Keep new behavior behind feature flags where feasible during rollout.
- On regression, disable affected command path while preserving existing indexed search and file discovery commands.

## Open Questions

- Which specific legacy grep flags are mandatory for v1 replacement status versus deferred compatibility?
- Should filesystem commands default to native path formatting or configurable output formatting for interoperability?
- What performance SLO thresholds define "production ready" for large repositories across all target platforms?
