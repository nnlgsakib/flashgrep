## MODIFIED Requirements

### Requirement: JSON-RPC protocol
The MCP server SHALL communicate using JSON-RPC 2.0 over a transport layer and MUST include additive machine-readable policy metadata in bootstrap-related responses without breaking existing clients. For path-aware tool calls, error responses MUST include deterministic typed not-found diagnostics when target paths are missing.

#### Scenario: Accept JSON-RPC requests
- **WHEN** a client sends a valid JSON-RPC request
- **THEN** the server SHALL parse and process the request

#### Scenario: Return JSON-RPC responses
- **WHEN** processing completes
- **THEN** the server SHALL return a JSON-RPC 2.0 response object

#### Scenario: Bootstrap responses include policy metadata
- **WHEN** a bootstrap method succeeds
- **THEN** the response MUST include machine-readable policy metadata describing preferred tools and fallback gating rules

#### Scenario: Missing path in tool call returns typed diagnostics
- **WHEN** a path-aware tool call targets a non-existent file or directory
- **THEN** the server MUST return error payload fields that explicitly identify not-found semantics for agent decision-making

#### Scenario: MCP exposes filesystem lifecycle tool methods
- **WHEN** an MCP client lists tools or invokes filesystem lifecycle operations
- **THEN** the server MUST expose and handle filesystem methods for create/read/write/list/stat/copy/move/remove with structured request/response semantics
