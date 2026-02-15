## 1. MCP Server Infrastructure

- [ ] 1.1 Implement `flashgrep mcp` command for MCP server mode
- [ ] 1.2 Add JSON-RPC 2.0 request/response handling
- [ ] 1.3 Implement MCP initialize method with capabilities
- [ ] 1.4 Add MCP tool listing endpoint
- [ ] 1.5 Implement MCP tool invocation routing
- [ ] 1.6 Add stdio transport support for MCP
- [ ] 1.7 Add error handling for MCP protocol violations

## 2. Skill Definition Generation

- [ ] 2.1 Create skill markdown template with frontmatter structure
- [ ] 2.2 Write skill content for `query` tool with usage examples
- [ ] 2.3 Write skill content for `files` tool with glob pattern examples
- [ ] 2.4 Write skill content for `symbol` tool with symbol lookup examples
- [ ] 2.5 Write skill content for `read_code` tool with continuation examples
- [ ] 2.6 Write skill content for `write_code` tool with precondition examples
- [ ] 2.7 Add "Do" and "Don't" examples for each tool
- [ ] 2.8 Create Keywords section with search and file operation terms
- [ ] 2.9 Add tool priority guidance and fallback rules
- [ ] 2.10 Implement skill generation command `flashgrep generate-skill`

## 3. OpenCode Integration

- [ ] 3.1 Implement `flashgrep install-opencode` command
- [ ] 3.2 Create OpenCode tool definition file template (TypeScript)
- [ ] 3.3 Add MCP server configuration to opencode.json
- [ ] 3.4 Handle existing OpenCode config (merge, don't overwrite)
- [ ] 3.5 Detect OpenCode installation and config paths
- [ ] 3.6 Implement `flashgrep uninstall-opencode` command
- [ ] 3.7 Remove tool definition file on uninstall
- [ ] 3.8 Remove MCP server entry from config on uninstall
- [ ] 3.9 Clean up empty directories after uninstall
- [ ] 3.10 Test OpenCode install/uninstall on clean system

## 4. Claude Code Integration

- [ ] 4.1 Implement `flashgrep install-claude` command
- [ ] 4.2 Add flashgrep to Claude Code marketplace
- [ ] 4.3 Create plugin manifest for Claude Code
- [ ] 4.4 Detect Claude Code installation
- [ ] 4.5 Show clear error if Claude Code not found
- [ ] 4.6 Implement `flashgrep uninstall-claude` command
- [ ] 4.7 Uninstall plugin from Claude Code
- [ ] 4.8 Remove from marketplace on uninstall
- [ ] 4.9 Test Claude Code install/uninstall workflow

## 5. Codex Integration

- [ ] 5.1 Implement `flashgrep install-codex` command
- [ ] 5.2 Add MCP server configuration for Codex
- [ ] 5.3 Append skill definition to ~/.codex/AGENTS.md
- [ ] 5.4 Detect if skill already exists (avoid duplicates)
- [ ] 5.5 Detect Codex installation
- [ ] 5.6 Implement `flashgrep uninstall-codex` command
- [ ] 5.7 Remove MCP configuration on uninstall
- [ ] 5.8 Remove skill from AGENTS.md on uninstall
- [ ] 5.9 Handle partial uninstalls gracefully
- [ ] 5.10 Test Codex install/uninstall workflow

## 6. Agent Detection and Auto-Install

- [ ] 6.1 Implement agent detection logic
- [ ] 6.2 Detect OpenCode installation
- [ ] 6.3 Detect Claude Code installation
- [ ] 6.4 Detect Codex installation
- [ ] 6.5 Implement `flashgrep install` (auto-detect all)
- [ ] 6.6 Install in all detected agents
- [ ] 6.7 Report which agents were configured
- [ ] 6.8 Show helpful message if no agents detected
- [ ] 6.9 Provide installation instructions per agent

## 7. Bootstrap Enhancement

- [ ] 7.1 Add skill metadata to bootstrap response
- [ ] 7.2 Include skill document reference in bootstrap
- [ ] 7.3 Derive tool priorities from skill context
- [ ] 7.4 Add skill-based examples to bootstrap response
- [ ] 7.5 Ensure skill metadata consistency across calls
- [ ] 7.6 Update bootstrap to reflect installed skill version
- [ ] 7.7 Test bootstrap with skill injection enabled

## 8. Error Handling and Edge Cases

- [ ] 8.1 Handle permission denied errors gracefully
- [ ] 8.2 Provide clear error messages for missing agents
- [ ] 8.3 Handle corrupted agent config files
- [ ] 8.4 Create config backups before modification
- [ ] 8.5 Handle concurrent install/uninstall operations
- [ ] 8.6 Validate JSON config before writing
- [ ] 8.7 Add rollback mechanism for failed installs
- [ ] 8.8 Handle agent version compatibility issues

## 9. Testing

- [ ] 9.1 Write unit tests for MCP server protocol
- [ ] 9.2 Write unit tests for skill generation
- [ ] 9.3 Write unit tests for OpenCode installer
- [ ] 9.4 Write unit tests for Codex installer
- [ ] 9.5 Create integration test for full install workflow
- [ ] 9.6 Test uninstall leaves system clean
- [ ] 9.7 Test idempotent installs (run twice)
- [ ] 9.8 Test with multiple agents installed
- [ ] 9.9 Verify skill content is correct and complete
- [ ] 9.10 Test bootstrap returns correct skill metadata

## 10. Documentation

- [ ] 10.1 Document `flashgrep install` command
- [ ] 10.2 Document per-agent install commands
- [ ] 10.3 Document `flashgrep uninstall` commands
- [ ] 10.4 Create troubleshooting guide
- [ ] 10.5 Document manual installation steps
- [ ] 10.6 Add examples of skill-based tool usage
- [ ] 10.7 Document config file locations per agent
- [ ] 10.8 Create migration guide from manual tool selection
