# Data Model: AI Capability Expansion and Efficiency

## Entity: AIScopeProfile

- Purpose: Declares where AI is permitted and expected value per workflow.
- Fields:
  - `scope_id` (string, required)
  - `workflow_type` (enum: `discovery`, `synthesis`, `planning`, required)
  - `enabled` (boolean, required)
  - `priority` (enum: `high`, `medium`, `low`, required)
  - `fallback_strategy` (enum: `deterministic_lexical`, required)
- Validation rules:
  - Scope MUST define fallback strategy.
  - Disabled scope MUST not route to AI execution.

## Entity: PromptBudgetRecord

- Purpose: Tracks token-budget planning and consumption per request.
- Fields:
  - `request_id` (string, required)
  - `budget_profile` (enum: `fast`, `balanced`, `deep`, required)
  - `budget_total` (integer, required)
  - `budget_system` (integer, required)
  - `budget_context` (integer, required)
  - `budget_memory` (integer, required)
  - `budget_response` (integer, required)
  - `tokens_used` (integer, required)
  - `reduction_applied` (boolean, required)
- Validation rules:
  - Budget components MUST sum to `budget_total`.
  - `tokens_used` MUST be <= `budget_total` for compliant requests.

## Entity: RoutingDecisionTrace

- Purpose: Records route outcome and reason metadata for each AI-capable request.
- Fields:
  - `request_id` (string, required)
  - `route_state` (enum: `allowed_native`, `allowed_ai`, `allowed_fallback`, `denied`, required)
  - `reason_code` (string, optional)
  - `confidence_level` (enum: `high`, `medium`, `low`, optional)
  - `fallback_gate_id` (string, optional)
  - `policy_hash` (string, required)
  - `prompt_version` (string, optional)
- Validation rules:
  - Denied/fallback decisions MUST include `reason_code`.
  - AI route decisions MUST include `prompt_version`.

## Entity: PromptPolicyRule

- Purpose: Represents deterministic constraints applied before AI execution.
- Fields:
  - `rule_id` (string, required)
  - `rule_type` (enum: `allow`, `deny`, `escalate`, required)
  - `condition` (string, required)
  - `reason_code` (string, required)
  - `remediation_hint` (string, optional)
  - `active` (boolean, required)
- Validation rules:
  - Active deny/escalate rules MUST provide typed reason code.
  - Rules MUST be evaluated in deterministic order.

## Entity: AISessionContextPack

- Purpose: Bounded, ordered context payload prepared for AI workflows.
- Fields:
  - `pack_id` (string, required)
  - `entries` (array, required) where each entry includes:
    - `source_path` (string)
    - `symbol_name` (string, optional)
    - `start_line` (integer)
    - `end_line` (integer)
    - `truncated_before` (boolean)
    - `truncated_after` (boolean)
    - `continuation_id` (string, optional)
  - `ordered_by` (enum: `relevance_then_path`, required)
- Validation rules:
  - Entry ordering MUST be deterministic.
  - Pack size MUST satisfy configured token budget.

## Relationships

- One `AIScopeProfile` can produce many `RoutingDecisionTrace` records.
- One request produces one `PromptBudgetRecord` and one `RoutingDecisionTrace`.
- One `RoutingDecisionTrace` may reference multiple `PromptPolicyRule` evaluations.
- One AI-assisted request uses one `AISessionContextPack`.

## State Transitions

### AI request lifecycle

`received` -> `policy_checked` -> (`allowed_native` | `allowed_ai` | `allowed_fallback` | `denied`) -> `completed`

### Budget lifecycle

`allocated` -> (`within_budget` | `reduced`) -> `reported`

### Prompt governance lifecycle

`version_selected` -> `policy_validated` -> (`executed` | `denied`) -> `audited`
