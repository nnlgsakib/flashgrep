## Context

The project is a Rust-based grep tool called "flashgrep". Currently, it's a standalone CLI tool. We need to add MCP (Model Context Protocol) support to make it accessible to coding agents like Claude, Codex, and OpenCode.

## Goals / Non-Goals

**Goals:**
- Implement an MCP server that exposes grep functionality
- Create MCP tool definitions for search operations
- Support standard MCP client interactions
- Create a skill for using the grep tool via MCP
- Maintain compatibility with existing CLI functionality

**Non-Goals:**
- Implementing a full MCP server from scratch
- Supporting all possible MCP extensions
- Changing the core grep functionality

## Decisions

### Decision 1: MCP Library Choice

**Choice:** Use existing MCP library (e.g., `mcp-server` crate)

**Rationale:** 
- Avoid reinventing the wheel
- Leverage existing community support
- Faster implementation time

**Alternatives considered:**
- Building custom MCP server (too time-consuming)
- Using other protocols (e.g., LSP) (not as widely adopted for coding agents)

### Decision 2: Tool Design

**Choice:** Create separate MCP tools for common grep operations

**Tools to implement:**
- `search`: Basic grep search with pattern and file filters
- `search-in-directory`: Search within specific directory
- `search-with-context`: Search with context lines
- `search-by-regex`: Regex-based search

**Rationale:**
- Each tool focuses on a specific use case
- Easier to maintain and extend
- Clear interface for coding agents

### Decision 3: Server Architecture

**Choice:** Integrate MCP server as an optional module

**Architecture:**
- CLI commands remain intact
- MCP server can be started with `flashgrep mcp` command
- Server listens on configurable port (default: 2000)
- Tools registered with MCP server on startup

**Rationale:**
- Maintains backward compatibility
- Optional MCP functionality doesn't affect CLI performance
- Clean separation of concerns

## Risks / Trade-offs

### Risk 1: Dependency Bloat

**Risk:** Adding MCP server dependency may increase binary size

**Mitigation:** Make MCP server an optional feature flag

### Risk 2: Performance Overhead

**Risk:** MCP server may introduce performance overhead

**Mitigation:** Keep MCP server lightweight and async
- Use tokio async runtime
- Process requests in parallel
- Minimize overhead in tool implementations