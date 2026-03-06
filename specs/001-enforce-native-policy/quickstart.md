# Quickstart: Validate Native Policy Enforcement

## 1) Baseline checks

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Expected result: baseline passes before policy-enforcement changes are merged.

## 2) Verify strict native routing

1. Initialize MCP session and inspect bootstrap payload.
2. Attempt a non-native route without a declared fallback gate.

Expected result:
- request is denied,
- denial payload includes typed `reason_code` and recovery guidance,
- session remains stable for subsequent requests.

## 3) Verify gated fallback admission

1. Submit request with valid fallback gate and reason code.
2. Repeat with invalid/unsupported reason code.

Expected result:
- valid gate is admitted and auditable,
- invalid reason code is denied with typed diagnostics.

## 4) Verify drift recovery

1. Simulate drift (policy metadata mismatch or missing field).
2. Trigger forced bootstrap reinjection.
3. Re-run native/fallback route checks.

Expected result:
- strict policy state is restored,
- routing outcomes remain deterministic after recovery.

## 5) Verify docs/skills parity

1. Confirm README policy/routing section matches runtime behavior.
2. Confirm `skills/SKILL.md` directives match enforced route order and fallback rules.
3. Confirm troubleshooting/release docs include typed denial and fallback semantics.

Expected result: guidance artifacts align with runtime contract and pass parity checks.

## Validation Record

- Date: 2026-03-06
- `cargo test`: pass
- `cargo clippy --all-targets -- -D warnings`: pass
- Verified strict policy denial for ungated fallback routes (`policy_denied` + typed `reason_code`).
- Verified gated fallback admission for valid `fallback_gate` and `fallback_reason_code` pairs.
- Verified policy drift detection (`policy_state_mismatch`) and force bootstrap recovery path.
