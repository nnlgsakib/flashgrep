# Contract: AI Routing, Token Budgets, and Prompt Governance

## Scope

This contract defines behavior for AI-assisted routing, token-aware prompt/context
management, and prompt-policy governance in Flashgrep CLI/MCP flows.

## AI Route Contract

### Required request controls

- Explicit AI mode/profile fields (for example: discovery/synthesis/planning intent,
  budget profile, and fallback preference).
- Optional policy metadata assertions (`policy_hash`, policy version marker).

### Route outcomes

- `allowed_native`
- `allowed_ai`
- `allowed_fallback`
- `denied`

### Required decision metadata

- `route_state`
- `reason_code` (for fallback/denied)
- `policy_hash`
- `prompt_version` (for AI route)
- `fallback_gate_id` (when fallback applies)

## Token Budget Contract

### Required budget fields

- `budget_total`
- `budget_system`
- `budget_context`
- `budget_memory`
- `budget_response`
- `tokens_used`
- `reduction_applied`

### Behavioral guarantees

- Requests over budget are reduced deterministically, not silently expanded.
- Responses include reduction/continuation signals when context is trimmed.
- Budget usage is reported in machine-readable metadata.

## Prompt Governance Contract

### Required governance fields

- `prompt_id`
- `prompt_version`
- `prompt_hash`
- `policy_rule_hits` (allow/deny/escalate)

### Denial contract

When denied by governance checks, payload MUST include:

- `ok = false`
- `error = policy_denied`
- `reason_code`
- `recovery_hint`

### Drift handling

- Policy/prompt mismatch MUST produce typed denial (`policy_state_mismatch` or
  equivalent) and remediation guidance.
- Forced reinjection/reload path MUST restore compliant session state.

## Fallback Contract

### Requirements

- AI fallback MUST be explicit and reason-coded.
- Fallback path MUST return deterministic outputs compatible with automation.

### Required fallback metadata

- `fallback_gate_id`
- `fallback_reason_code`
- `route_state = allowed_fallback`

## Docs/Skills Parity Contract

### Release gate requirements

- README documents AI mode controls, budgets, and typed denial/fallback semantics.
- `skills/SKILL.md` reflects current route order, governance constraints, and
  token-budget usage guidance.
- Troubleshooting/release docs include parity checks for runtime metadata fields.
