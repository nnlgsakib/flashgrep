## Why

The current grep tool lacks proper MCP (Model Context Protocol) support, making it difficult for coding agents like Claude, Codex, and OpenCode to use it programmatically. This limits the tool's accessibility and integration capabilities with modern AI coding workflows.

## What Changes

- Add MCP protocol support to the grep tool
- Create an MCP server that exposes grep functionality as MCP tools
- Implement MCP tool definitions for grep operations
- Create a skill for using the grep tool via MCP
- Ensure compatibility with standard MCP clients

## Capabilities

### New Capabilities

- `mcp-server`: MCP server implementation that exposes grep functionality
- `mcp-tools`: MCP tool definitions for grep operations
- `mcp-skill`: Skill for using the grep tool via MCP

### Modified Capabilities

<!-- No existing capabilities are being modified -->

## Impact

- Affected files: Main application code, new MCP server module, tool definitions
- New dependencies: MCP protocol library
- Integration points: AI coding agent workflows

## Use Cases

1. AI coding agents can use the grep tool to search codebases
2. Integration with MCP-compatible tools and platforms
3. Programmatic access to grep functionality via MCP protocol