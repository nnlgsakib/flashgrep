# Contract: Native Policy Enforcement and Drift Recovery

## Scope

This contract defines session policy enforcement behavior for MCP requests,
including native routing, fallback gate admission, typed denials, and drift
recovery expectations.

## Bootstrap Contract

### Required fields

- `status` (`injected` | `already_injected`)
- `payload_source` (`embedded` | `repo_override`)
- `policy_metadata.policy_strength`
- `policy_metadata.search_routing`
- `policy_metadata.fallback_rules`
- `policy_metadata.fallback_gate_ids`

### Behavioral guarantees

- Session bootstrap is idempotent within a running session.
- Missing or invalid trigger input returns typed error behavior.
- Legacy bootstrap fields remain available for compatibility.

## Routing Enforcement Contract

### Required request behavior

- Non-native route attempts without valid gate are denied.
- Valid fallback gate + reason code permits constrained fallback route.
- Decision outcomes are deterministic for identical request + session state.

### Route outcomes

- `allowed_native`
- `allowed_fallback`
- `denied`

### Required denial payload

- `ok = false`
- `error = policy_denied` (or equivalent typed denial)
- `reason_code`
- `recovery_hint`

## Drift Detection and Recovery Contract

### Drift triggers

- Policy metadata missing or malformed
- Unsupported fallback reason code
- Policy state mismatch detected during route evaluation

### Recovery behavior

- Force bootstrap reinjection restores strict policy state.
- Post-recovery requests enforce native-first routing consistently.

## Docs/Skills Parity Contract

### Required parity checks

- README native-routing guidance reflects current enforcement contract.
- `skills/SKILL.md` route/workflow directives match policy metadata.
- Troubleshooting and release checklists include typed denial + fallback semantics.

### Release gate

- All parity checks MUST pass before release readiness completion.
