## Why

AI agents like OpenCode, Claude Code, and others need clear documentation on how to effectively use the flashgrep MCP server for code search. Currently, there's no comprehensive guide teaching AI agents the best practices, available tools, and optimal query patterns. Adding a skill.md file will enable AI agents to leverage flashgrep's full potential for code analysis, navigation, and understanding.

## What Changes

- **Add SKILL.md file**: Create comprehensive documentation for AI agents at `.opencode/skills/flashgrep-mcp/SKILL.md`
- **Document all MCP tools**: Explain each tool (query, get_slice, get_symbol, list_files, stats) with examples
- **Provide usage patterns**: Common search patterns and best practices for AI agents
- **Include example workflows**: Step-by-step examples of how to use flashgrep for different tasks
- **Document error handling**: How to handle and recover from MCP errors
- **Add configuration guide**: How to configure flashgrep MCP for optimal performance

## Capabilities

### New Capabilities
- `ai-agent-documentation`: Documentation for AI agents to use flashgrep MCP effectively

### Modified Capabilities
- None

## Impact

- **New File**: `.opencode/skills/flashgrep-mcp/SKILL.md`
- **No Breaking Changes**: Pure documentation addition
- **AI Agent Experience**: AI agents will better understand how to use flashgrep for code search
- **User Benefit**: More effective code analysis and navigation when using AI agents with flashgrep
