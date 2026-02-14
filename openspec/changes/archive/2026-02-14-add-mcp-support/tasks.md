## 1. MCP Server Implementation

- [x] 1.1 Add MCP server dependency to Cargo.toml
- [x] 1.2 Create MCP server module
- [x] 1.3 Implement server startup and shutdown logic
- [x] 1.4 Add CLI command for starting MCP server
- [x] 1.5 Implement server configuration (port, log level)
- [x] 1.6 Add health check endpoint

## 2. MCP Tools Implementation

- [x] 2.6 Add tool parameter validation
- [x] 2.7 Implement error handling for tool operations

## 3. MCP Skill Implementation

- [x] 3.4 Add error handling for skill operations
- [x] 3.5 Test skill with MCP clients

## 4. CLI Command Separation

- [x] 4.7 Separate commands:
  - ✅ `start` - Only starts file watcher
  - ✅ `stop` - Stops file watcher (via process termination)
  - ✅ `mcp` - Only starts MCP server
  - ✅ `index` - Performs initial indexing

## 5. Integration and Testing

- [x] 4.1 Test MCP server startup and shutdown
- [x] 4.2 Test all MCP tools with various inputs
- [x] 4.3 Test integration with existing CLI functionality
- [x] 4.4 Test with MCP client (e.g., OpenCode)
- [x] 4.5 Performance testing
- [x] 4.6 Fix any issues found in testing