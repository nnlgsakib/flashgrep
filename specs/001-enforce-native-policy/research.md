# Research: Enforce Native Policy Routing

## Decision 1: Enforce deny-by-default routing

- Decision: Route evaluation defaults to deny for non-native actions unless an
  explicit fallback gate is present and valid.
- Rationale: Default-deny prevents silent policy bypass and makes compliance
  deterministic.
- Alternatives considered:
  - Allow-by-default with blocklist (rejected: porous and hard to maintain)
  - Best-effort heuristics (rejected: non-deterministic outcomes)

## Decision 2: Use typed, machine-actionable policy violations

- Decision: Return structured violation classes and reason codes for denied or
  constrained requests.
- Rationale: Typed responses support automated recovery, observability, and stable
  CI checks.
- Alternatives considered:
  - Free-form error text only (rejected: not automation-friendly)
  - Single generic denial code (rejected: poor diagnostics)

## Decision 3: Require explicit fallback gate objects

- Decision: Treat fallback as explicit, auditable policy records with gate identity,
  condition, and reason code.
- Rationale: Fallback is the highest-risk bypass path; explicit gates preserve
  operational flexibility while retaining control.
- Alternatives considered:
  - Implicit fallback on failure (rejected: drift-prone)
  - Disable fallback entirely (rejected: operationally brittle)

## Decision 4: Make drift detection fail-safe per request cycle

- Decision: Validate policy state/hash per request and trigger force reinjection when
  integrity checks fail.
- Rationale: Immediate fail-safe checks keep sessions consistent and bounded.
- Alternatives considered:
  - Session-start-only validation (rejected: misses mid-session drift)
  - Periodic polling (rejected: delayed response)

## Decision 5: Keep policy state canonical and idempotent

- Decision: Reinjection uses canonical payload ordering and idempotent state updates.
- Rationale: Prevents policy accretion and conflicting layered directives.
- Alternatives considered:
  - Incremental append-only policy prompts (rejected: ambiguity/drift)
  - Manual ad hoc repair (rejected: unreliable)

## Decision 6: Gate docs/skills parity in release flow

- Decision: Add parity checks that assert runtime metadata, README guidance, and
  `skills/SKILL.md` semantics remain synchronized.
- Rationale: Policy behavior must match user and agent guidance to prevent repeated
  off-policy routing.
- Alternatives considered:
  - Manual checklist only (rejected: easy to miss)
  - Runtime tests without doc checks (rejected: stale guidance risk)

## Decision 7: Preserve backward-compatible bootstrap contract

- Decision: Keep legacy bootstrap fields while extending strict policy metadata.
- Rationale: Maintains compatibility for existing clients while enabling stronger
  enforcement.
- Alternatives considered:
  - Breaking metadata rewrite (rejected: migration risk)
  - No metadata evolution (rejected: insufficient enforcement expressiveness)
