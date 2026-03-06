# Data Model: Enforce Native Policy Routing

## Entity: PolicySessionState

- Purpose: Represents current enforcement state for an MCP session.
- Fields:
  - `session_id` (string, required)
  - `bootstrap_state` (enum: `injected`, `already_injected`, `missing`, required)
  - `policy_strength` (enum: `strict`, required)
  - `policy_hash` (string, required)
  - `policy_version` (string, required)
  - `payload_source` (enum: `embedded`, `repo_override`, required)
- Validation rules:
  - Strict mode sessions MUST include valid `policy_hash` and `policy_version`.
  - `payload_source` MUST be explicit in every bootstrap result.

## Entity: RoutingDecision

- Purpose: Records deterministic route outcome for a request.
- Fields:
  - `request_id` (string, required)
  - `route_state` (enum: `allowed_native`, `allowed_fallback`, `denied`, required)
  - `tool_name` (string, required)
  - `reason_code` (string, optional)
  - `fallback_gate_id` (string, optional)
  - `message` (string, optional)
- Validation rules:
  - `allowed_fallback` MUST include `fallback_gate_id` and `reason_code`.
  - `denied` MUST include typed `reason_code` and actionable message.

## Entity: FallbackGateRecord

- Purpose: Defines permissible fallback behavior.
- Fields:
  - `gate_id` (string, required)
  - `condition` (string, required)
  - `allowed_tools` (array<string>, required)
  - `reason_code` (string, required)
  - `active` (boolean, required)
- Validation rules:
  - `reason_code` MUST match one of declared policy reason codes.
  - Gate MUST be inactive unless condition evaluates true.

## Entity: PolicyViolationEvent

- Purpose: Audit record for rejected or constrained policy actions.
- Fields:
  - `event_id` (string, required)
  - `session_id` (string, required)
  - `request_id` (string, required)
  - `violation_type` (string, required)
  - `reason_code` (string, required)
  - `recovery_hint` (string, optional)
  - `timestamp` (datetime string, required)
- Validation rules:
  - Violation events MUST be generated for every denied route.
  - Event payload MUST include deterministic `reason_code`.

## Entity: GuidanceParityRecord

- Purpose: Tracks synchronization between runtime policy and docs/skills.
- Fields:
  - `artifact` (enum: `README`, `SKILL`, `docs`, required)
  - `policy_fields_checked` (array<string>, required)
  - `parity_status` (enum: `pass`, `fail`, required)
  - `mismatch_summary` (string, optional)
- Validation rules:
  - Release gate requires all parity records to be `pass`.

## Relationships

- One `PolicySessionState` can produce many `RoutingDecision` records.
- One `RoutingDecision` may reference one `FallbackGateRecord`.
- One denied `RoutingDecision` creates one `PolicyViolationEvent`.
- One release check produces multiple `GuidanceParityRecord` entries.

## State Transitions

### Policy session lifecycle

`missing` -> `injected` -> `already_injected`

- On drift or integrity failure: `injected|already_injected` -> `missing` -> `injected`

### Routing decision lifecycle

`received` -> `evaluated` -> (`allowed_native` | `allowed_fallback` | `denied`)
