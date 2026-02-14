## Why

Agents currently need manual skill setup before they consistently prefer Flashgrep tools, which leads to fallback usage of generic grep/glob flows and higher token cost. We need a native, one-command bootstrap (`flashgrep-init` / `fgrep-boot`) that injects Flashgrep skill guidance at session start so agents use indexed search and efficient read/write tools immediately.

## What Changes

- Add a native MCP bootstrap command flow that accepts init triggers (for example `flashgrep-init` / `fgrep-boot`) and returns skill-injection payloads for the connected agent session.
- Integrate skill delivery from the in-repo `skills/SKILL.md` source so no external/manual skill file loading is required.
- Add startup guidance in the bootstrap payload that instructs agents to prefer Flashgrep tools (`query`, `files`, `symbol`, `read_code`, `write_code`) over generic grep/glob workflows where applicable.
- Define session behavior for skill injection lifecycle (initial inject, idempotent reinject, and error handling when skill content is unavailable).
- Update MCP documentation and agent guidance to reflect the native bootstrap + auto-injection workflow.

## Capabilities

### New Capabilities
- `agent-skill-bootstrap`: Session bootstrap endpoint/command that injects Flashgrep skill content into AI agents at startup.

### Modified Capabilities
- `mcp-server`: Extend MCP behavior to support skill bootstrap requests and structured skill-injection responses.
- `ai-agent-documentation`: Update documented recommended agent flow to use native bootstrap and Flashgrep-first tool selection.

## Impact

- Affected code: MCP request routing/handlers, skill loading/injection module(s), and command aliases for init triggers.
- Affected docs: `skills/SKILL.md` usage guidance and user-facing README/agent setup sections.
- Runtime impact: agents can start with Flashgrep-native guidance immediately, reducing token-heavy fallback usage patterns.
