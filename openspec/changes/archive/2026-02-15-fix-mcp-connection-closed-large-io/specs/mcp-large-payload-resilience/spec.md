## ADDED Requirements

### Requirement: Large MCP payloads do not terminate active sessions
The MCP server MUST preserve the underlying client session when handling large read and write tool operations and MUST return structured tool-level failures instead of transport connection closure.

#### Scenario: Large read request exceeds practical response size
- **WHEN** a client calls `read_code` or `get_slice` for a range that would exceed server response limits
- **THEN** the server MUST keep the session open and return a structured error or deterministic truncated response with continuation guidance

#### Scenario: Large write request exceeds replacement size limit
- **WHEN** a client calls `write_code` with replacement content larger than the configured maximum
- **THEN** the server MUST reject the request with structured size-limit metadata and MUST NOT close the transport connection

### Requirement: Size-limit failures are machine-actionable
The MCP server MUST return machine-readable metadata for large-payload rejections so clients can retry with smaller chunks.

#### Scenario: Error contains observed and allowed sizes
- **WHEN** a request or response payload breaches a defined size bound
- **THEN** the server MUST return limit and observed size fields and an error type suitable for programmatic handling

#### Scenario: Error includes chunking guidance
- **WHEN** a size-limit error is returned for read or write operations
- **THEN** the response MUST include actionable next-step guidance for chunked continuation (for example next line range or max replacement size)
