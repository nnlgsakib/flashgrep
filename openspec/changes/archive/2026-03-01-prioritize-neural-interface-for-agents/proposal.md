## Why

AI agents currently default to programmatic/lexical search patterns, which can require multiple retries before they find the right files. Prioritizing the neural interface first improves intent-level discovery and reduces unnecessary tool churn for agent workflows.

## What Changes

- Add explicit agent skill policy that routes agent search requests to neural retrieval first, then falls back to programmatic/lexical search when needed.
- Add deterministic fallback gates so agents only use programmatic search as second priority when neural mode is unavailable, low-confidence, or query-specific constraints require lexical matching.
- Update bootstrap policy metadata to include neural-first ordering and compliance fields for observability.
- Update agent-facing documentation/skill guidance to enforce neural-first workflow and clarify when to use fallback search modes.

## Capabilities

### New Capabilities
- `agent-neural-routing-policy`: Defines neural-first agent routing semantics and typed fallback gates for programmatic second-choice behavior.

### Modified Capabilities
- `agent-tool-priority-policy`: Update tool priority policy to include neural-first routing order and fallback reason codes.
- `agent-skill-bootstrap`: Update bootstrap guidance payload to instruct agents to use neural interface first for discovery tasks.
- `ai-agent-documentation`: Update documentation to describe neural-first workflows and deterministic fallback criteria.

## Impact

- Affected code: bootstrap policy payload generation, policy metadata schema, and agent guidance docs.
- Affected runtime behavior: AI agents receive neural-first tool ordering at initialization.
- Compatibility: Existing programmatic search remains available as explicit second priority fallback.
- Observability: Adds compliance metadata to diagnose whether neural-first policy is being followed.
