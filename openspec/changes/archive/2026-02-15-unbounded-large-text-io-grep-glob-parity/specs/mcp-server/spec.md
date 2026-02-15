## MODIFIED Requirements

### Requirement: JSON-RPC protocol
The MCP server SHALL communicate using JSON-RPC 2.0 over a transport layer and MUST support multi-part continuation responses for large logical operations without requiring one-message completion.

#### Scenario: Accept JSON-RPC requests
- **WHEN** a client sends a valid JSON-RPC request
- **THEN** the server SHALL parse and process the request

#### Scenario: Return JSON-RPC responses
- **WHEN** processing completes
- **THEN** the server SHALL return a JSON-RPC 2.0 response object

#### Scenario: Large logical operation returns continuation contract
- **WHEN** a request cannot be fully represented in one packet-safe response
- **THEN** the server MUST return structured continuation metadata that allows the client to continue until full completion

### Requirement: Transport layer
The MCP server SHALL support multiple transport mechanisms and MUST preserve active session continuity for multi-part large operations.

#### Scenario: TCP transport
- **WHEN** running on any platform
- **THEN** it SHALL accept connections on localhost:7777 by default

#### Scenario: Unix socket transport
- **WHEN** running on Linux or Mac
- **THEN** it SHALL support Unix domain sockets at `.flashgrep/mcp.sock`

#### Scenario: Multi-part large operation does not terminate session
- **WHEN** a client runs continuation-based read/write/query/glob operations over many chunks
- **THEN** the transport session MUST remain active and process successive continuation calls deterministically
