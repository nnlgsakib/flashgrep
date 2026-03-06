# Quickstart: Validate AI Expansion and Efficiency

## 1) Baseline validation

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Expected result: baseline is green before AI-expansion changes.

## 2) Validate smarter AI-first discovery

1. Run representative intent-style discovery requests with AI mode enabled.
2. Confirm route metadata and top results quality.
3. Simulate AI-unavailable and low-confidence conditions.

Expected result:
- AI route provides higher-quality targets for eligible requests.
- Fallback route remains deterministic with typed reason codes.

## 3) Validate token-budget enforcement

1. Run requests under each budget profile (`fast`, `balanced`, `deep`).
2. Submit over-budget contexts.
3. Verify reduction/continuation metadata.

Expected result:
- Requests respect configured budget limits.
- Over-budget contexts are reduced deterministically.
- Budget usage and reduction markers are always reported.

## 4) Validate prompt governance and policy checks

1. Execute compliant AI requests.
2. Execute intentionally non-compliant requests.
3. Validate typed denial payload fields and recovery hints.

Expected result:
- Compliant requests execute with prompt version/hash metadata.
- Non-compliant requests return `policy_denied` with typed `reason_code`.

## 5) Validate drift recovery

1. Simulate policy or prompt metadata mismatch.
2. Trigger force bootstrap/reload path.
3. Re-run AI route checks.

Expected result:
- Drift is detected with typed diagnostics.
- Force recovery restores compliant behavior in one cycle.

## 6) Validate docs/skills parity

1. Check README AI mode and fallback guidance.
2. Check `skills/SKILL.md` for updated governance and token-budget directives.
3. Run parity tests for runtime fields vs docs/skills.

Expected result: runtime behavior and guidance artifacts remain synchronized.

## Validation record (2026-03-06)

- `cargo test`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- AI query route checks: PASS (`allowed_ai`, deterministic `allowed_fallback`, typed `policy_denied`)
- Budget telemetry checks: PASS (`tokens_used`, `reduction_applied`, `continuation_id` present)
- Docs/skills parity checks: PASS (`README.md`, `skills/SKILL.md`, troubleshooting/readiness docs)
