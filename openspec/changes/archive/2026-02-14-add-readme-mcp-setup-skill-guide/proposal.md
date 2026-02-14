## Why

The README currently documents the MCP API methods but does not provide a complete, copy-paste MCP client setup example for `flashgrep mcp-stdio` or clear guidance on where to find skill files. Users need one place to quickly configure MCP clients and discover the skill documentation.

## What Changes

- Add a dedicated README section for MCP client setup using stdio transport with a full JSON config example for Flashgrep.
- Add explicit README guidance on agent-agnostic skill locations, with `skills/SKILL.md` as the generic path and `.opencode/skills/flashgrep-mcp/SKILL.md` as an optional OpenCode-managed path.
- Add step-by-step setup and verification notes so users can validate their MCP integration quickly.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `ai-agent-documentation`: Expand requirements to include complete MCP stdio setup documentation and skill discovery guidance in README-level docs.

## Impact

- **Docs updated**: `README.md`
- **Specs updated**: `openspec/changes/add-readme-mcp-setup-skill-guide/specs/ai-agent-documentation/spec.md`
- No runtime behavior changes, APIs, or dependencies.
