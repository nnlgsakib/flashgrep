## ADDED Requirements

### Requirement: MCP Server Startup
The system SHALL provide an MCP server that can be started via CLI command.

#### Scenario: Start MCP server
- **WHEN** user runs `flashgrep mcp`
- **THEN** MCP server starts and listens on default port 2000
- **AND** server logs startup information including listening port

#### Scenario: Start MCP server on custom port
- **WHEN** user runs `flashgrep mcp --port 3000`
- **THEN** MCP server starts and listens on port 3000
- **AND** server logs startup information with custom port

#### Scenario: MCP server shutdown
- **WHEN** user presses Ctrl+C
- **THEN** MCP server shuts down gracefully
- **AND** server logs shutdown information

### Requirement: MCP Server Health Check
The system SHALL provide a health check endpoint for the MCP server.

#### Scenario: Health check responds successfully
- **WHEN** MCP server receives a health check request
- **THEN** server responds with 200 OK status
- **AND** response contains health status information

### Requirement: MCP Tool Registration
The system SHALL register all available grep tools with the MCP server on startup.

#### Scenario: Tools registered on startup
- **WHEN** MCP server starts successfully
- **THEN** all grep tools are registered with the MCP server
- **AND** server logs registered tool names

### Requirement: MCP Server Configuration
The system SHALL allow configuring MCP server settings via command-line options.

#### Scenario: Set log level
- **WHEN** user runs `flashgrep mcp --log-level debug`
- **THEN** MCP server starts with debug log level
- **AND** detailed debug information is logged