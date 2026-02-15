## Why

Agents still bypass Flashgrep skill guidance and fall back to native read/write/search tools, causing inconsistent behavior, higher token use, and weaker index-first performance. The project needs stronger, production-ready policy and bootstrap behavior that reliably steers agents to Flashgrep tools first.

## What Changes

- Strengthen skill guidance so agent tool selection behavior is explicit, strict, and optimized for Flashgrep-first execution.
- Add enforceable policy semantics in bootstrap payloads so clients receive unambiguous tool-priority directives.
- Define deterministic fallback rules for when Flashgrep tools are unavailable or insufficient, minimizing uncontrolled native-tool usage.
- Add concise but high-control operational playbooks in `skills/SKILL.md` for read/write/search workflows.
- Add validation and observability expectations to verify that policy injection is active and behaving as intended.

## Capabilities

### New Capabilities
- `agent-tool-priority-policy`: Defines enforceable tool-priority policy contract for agent sessions (Flashgrep-first with explicit fallback gates).

### Modified Capabilities
- `agent-skill-bootstrap`: Expand bootstrap semantics to include strict policy metadata and stable status signaling for enforcement mode.
- `ai-agent-documentation`: Update skill and docs requirements for optimized, high-compliance guidance that drives consistent tool behavior.
- `mcp-server`: Ensure bootstrap and related MCP responses carry policy metadata needed by clients to enforce preferred tool routing.

## Impact

- Affected code: bootstrap payload generation, MCP method response shaping, and skill/documentation assets.
- Affected APIs: `bootstrap_skill` and alias methods (`flashgrep-init`, `fgrep-boot`, etc.) response contract.
- Affected operations: agent workflow consistency, token efficiency, and tool selection observability during sessions.
