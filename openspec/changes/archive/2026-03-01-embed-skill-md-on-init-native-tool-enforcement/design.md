## Context

Flashgrep currently injects skill guidance by reading `skills/SKILL.md` from the repository at runtime. In practice this fails in several environments (wrong working directory, missing repo mirror, containerized agents without skill files), which results in degraded behavior where agents default back to native tools. The requested behavior is stronger: ship the canonical skill in the binary, inject it during init, and make Flashgrep-first routing feel native and deterministic.

## Goals / Non-Goals

**Goals:**
- Make bootstrap succeed without requiring any filesystem skill file.
- Ensure init-time payload always includes canonical Flashgrep-first policy guidance.
- Preserve backward compatibility for existing bootstrap endpoints and aliases.
- Add deterministic policy metadata to support client-side enforcement and troubleshooting.
- Provide strict fallback gates so native tool usage is allowed only for documented reasons.

**Non-Goals:**
- Building a vendor-specific prompt jailbreak system beyond policy and routing guidance.
- Removing all support for repository-local skill overrides (can remain as optional override behavior if explicitly enabled).
- Rewriting MCP protocol surface area unrelated to bootstrap and tool routing policy.

## Decisions

### Decision: Embed canonical skill text at compile time
Use compile-time embedding (`include_str!`) for the canonical bootstrap skill payload. This guarantees availability even when repository files are missing.

Alternatives considered:
- Runtime file read only: rejected due to reliability failures.
- Network fetch of skill content: rejected due to latency/offline risk and nondeterminism.

### Decision: Init path always prefers embedded payload
Bootstrap/init handlers will use embedded payload as source-of-truth by default. Optional repo-file override can be gated behind explicit configuration and checks, but missing file must not fail bootstrap.

Alternatives considered:
- Dual-source merge by default: rejected for complexity and nondeterministic ordering.
- Repo-file preferred, embedded fallback: rejected because it keeps the current failure mode primary.

### Decision: Enforce strict Flashgrep-first policy metadata
Bootstrap responses will include explicit enforcement metadata (`policy_strength`, preferred tool families, fallback gates, compliance hints) and a marker that policy injection occurred at init.

Alternatives considered:
- Soft advisory text only: rejected; too easy for clients/agents to ignore.
- Hard protocol rejection of non-Flashgrep tools: rejected as overly invasive for backward compatibility.

### Decision: Deterministic observability for troubleshooting
Return typed bootstrap diagnostics indicating payload source (`embedded`, `repo_override`), injection state, and any fallback reason. This enables reliable debugging when agents drift from policy.

Alternatives considered:
- Omit source metadata for simplicity: rejected; root-cause analysis becomes opaque.

## Risks / Trade-offs

- [Embedded payload can drift from docs] -> Mitigation: add tests/checks that validate embedded content hash against canonical source in repo.
- [Stricter policy may reduce flexibility for some agents] -> Mitigation: expose explicit fallback gates and typed reasons rather than blanket denial.
- [Optional repo override can reintroduce nondeterminism] -> Mitigation: keep embedded default, require explicit opt-in flag, and emit source metadata.

## Migration Plan

1. Introduce embedded canonical skill constant and wire bootstrap/init handlers to use it.
2. Add policy metadata extensions for source, enforcement mode, fallback gates, and compliance state.
3. Keep existing bootstrap tool names/aliases and response compatibility fields.
4. Add tests for init injection success without `skills/SKILL.md`, idempotency, and strict policy metadata.
5. Update docs and troubleshooting to reflect embedded-default behavior.

Rollback strategy:
- Revert to previous bootstrap payload builder while keeping compatibility fields.
- Guard new strict policy metadata behind a compatibility flag if client regressions are detected.

## Open Questions

- Should repo override be enabled by default in development only, or fully opt-in everywhere?
- Should policy metadata include an explicit machine-readable severity for routing violations?
- Do we want a startup warning when embedded and repo skill content diverge (if override is enabled)?
