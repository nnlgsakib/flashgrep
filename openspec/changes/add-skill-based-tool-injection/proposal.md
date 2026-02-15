## Why

Currently, users must manually specify which flashgrep tool to use for each operation. This is inefficient and requires users to know the exact tool names and when to use them. Modern AI agents (OpenCode, Claude Code, Codex) support skill-based tool injection where tools register themselves with the agent along with contextual guidance on when to use them. Flashgrep needs this capability so AI agents can automatically select the appropriate flashgrep tools based on context, just like mgrep does.

## What Changes

- **Add `install` CLI command**: Install flashgrep as an MCP tool/skill in the user's AI agent (OpenCode, Claude Code, Codex)
- **Add `uninstall` CLI command**: Remove flashgrep from the AI agent configuration
- **Create skill definition**: Write a skill markdown document describing when to use flashgrep tools (query, files, symbol, read_code, write_code) with do/don't examples
- **Auto-register with MCP**: Register flashgrep as an MCP server in the agent's configuration
- **Context-aware tool selection**: AI agents will automatically know to use flashgrep instead of generic grep/glob/read tools

## Capabilities

### New Capabilities
- `agent-tool-installer`: CLI commands to install/uninstall flashgrep as an AI agent tool with automatic MCP registration
- `skill-definition-generator`: Generate skill markdown documents describing tool usage patterns and contextual guidance

### Modified Capabilities
- `agent-skill-bootstrap`: Enhance bootstrap to support skill-based tool injection and automatic tool selection policies

## Impact

- New CLI commands: `flashgrep install` and `flashgrep uninstall`
- Support for multiple AI agent platforms (OpenCode, Claude Code, Codex)
- Changes to bootstrap mechanism to include skill-based guidance
- Users no longer need to manually specify which flashgrep tool to use
- Better integration with modern AI agent workflows
