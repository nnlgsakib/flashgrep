## MODIFIED Requirements

### Requirement: JSON-RPC protocol
The MCP server SHALL communicate using JSON-RPC 2.0 over a transport layer and MUST convert oversized or invalid large-IO operations into structured JSON-RPC-compatible error responses instead of dropping the connection.

#### Scenario: Accept JSON-RPC requests
- **WHEN** a client sends a valid JSON-RPC request
- **THEN** the server SHALL parse and process the request

#### Scenario: Return JSON-RPC responses
- **WHEN** processing completes
- **THEN** the server SHALL return a JSON-RPC 2.0 response object

#### Scenario: Oversized MCP tool payload returns structured error
- **WHEN** request or response sizing for a tool operation exceeds configured limits
- **THEN** the server MUST return a structured error payload suitable for retry/chunking and MUST keep the session transport alive

### Requirement: Transport layer
The MCP server SHALL support multiple transport mechanisms and MUST preserve transport stability during large read/write tool operations by isolating per-request failures from session lifecycle.

#### Scenario: TCP transport
- **WHEN** running on any platform
- **THEN** it SHALL accept connections on localhost:7777 by default

#### Scenario: Unix socket transport
- **WHEN** running on Linux or Mac
- **THEN** it SHALL support Unix domain sockets at `.flashgrep/mcp.sock`

#### Scenario: Large IO tool failure does not close transport session
- **WHEN** a large `read_code`, `get_slice`, or `write_code` call fails validation or hits payload limits
- **THEN** the transport connection MUST remain open and the server MUST continue processing subsequent requests
