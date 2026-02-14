## Context

Flashgrep already documents MCP server methods in `README.md`, but setup guidance is fragmented for users configuring MCP clients with stdio transport. The repository also has multiple skill documentation locations (`skills/SKILL.md` and `.opencode/skills/flashgrep-mcp/SKILL.md`) that are not explained together in one onboarding flow.

## Goals / Non-Goals

**Goals:**
- Add a complete, copy-paste MCP client configuration example for Flashgrep stdio mode in `README.md`.
- Add a short setup flow that includes indexing, launching `flashgrep mcp-stdio`, and validation steps.
- Add explicit guidance on skill file discovery and when to use each skill path.

**Non-Goals:**
- No changes to MCP server runtime behavior, protocols, or CLI flags.
- No new MCP methods or changes to JSON-RPC payload schemas.
- No changes to indexing logic or search behavior.

## Decisions

- **Decision: Document stdio-first setup in README**
  - **Rationale:** MCP clients commonly use stdio process launch; showing this first reduces setup failures.
  - **Alternative considered:** Keep only TCP-based setup notes.
  - **Why not alternative:** TCP-only guidance does not match most tool integrations that expect a command-based process.

- **Decision: Include full JSON config example using `flashgrep mcp-stdio` and `RUST_LOG=info`**
  - **Rationale:** Users requested a full block that can be copied directly.
  - **Alternative considered:** Provide partial snippets.
  - **Why not alternative:** Partial snippets increase ambiguity and user error.

- **Decision: Add explicit, agent-agnostic skill discovery section in README**
  - **Rationale:** Users need to quickly find the right skill docs for any coding agent, not just one client.
  - **Alternative considered:** Link only one skill path.
  - **Why not alternative:** Different workflows rely on different locations; both should be discoverable.

## Risks / Trade-offs

- **[Risk]** README duplication across sections could drift over time  
  **Mitigation:** Keep MCP setup content in one dedicated section and cross-link from API section.

- **[Risk]** Users may assume `RUST_LOG=info` is required in production  
  **Mitigation:** Clarify it is optional and mainly useful for troubleshooting.

- **[Trade-off]** More onboarding detail increases README length  
  **Mitigation:** Use concise headings and a compact step-by-step format.

## Migration Plan

1. Update README with a new MCP setup section and full JSON example.
2. Add skill discovery subsection with both skill paths and usage guidance.
3. Validate markdown formatting and path references.
4. Keep existing API method docs and add cross-links instead of duplication.

Rollback: revert README edits if docs regression is detected.

## Open Questions

- Should we include one additional example for n8n MCP client wiring in README, or keep README tool-agnostic and defer client-specific examples to skill docs?
