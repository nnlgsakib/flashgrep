## Why

The current skill injection flow is unreliable: agents may fail to locate `SKILL.md` and silently fall back to native tool preferences. This prevents consistent Flashgrep-first behavior and weakens automation guarantees.

## What Changes

- Embed the canonical skill payload in the binary and inject it automatically during agent initialization.
- Remove hard dependency on repository-local `skills/SKILL.md` lookup for initial bootstrap success.
- Add strict startup policy signaling that requires Flashgrep-native tool routing before any fallback behavior.
- Add deterministic failure and observability paths when skill injection cannot be applied.
- Update docs and operational guidance for the new embedded bootstrap model.

## Capabilities

### New Capabilities
- `embedded-skill-bootstrap`: Ship and serve an in-binary skill document that is injected at init time without filesystem prerequisites.

### Modified Capabilities
- `agent-skill-bootstrap`: Change bootstrap source-of-truth and initialization flow to prefer embedded payload over repo file discovery.
- `agent-tool-priority-policy`: Strengthen policy behavior so Flashgrep tools are treated as native-first and fallback requires explicit gated conditions.
- `ai-agent-documentation`: Update docs to describe embedded injection behavior, guarantees, and troubleshooting.

## Impact

- Affects MCP bootstrap handling, skill payload assembly, startup policy metadata, and agent tool-selection guidance.
- Adds tests for init-time embedded injection, missing-file fallback behavior, and strict policy enforcement.
- Updates README/skill docs and troubleshooting guidance to reflect embedded bootstrap architecture.
