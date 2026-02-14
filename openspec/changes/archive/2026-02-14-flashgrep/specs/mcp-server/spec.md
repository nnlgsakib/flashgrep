## ADDED Requirements

### Requirement: JSON-RPC protocol
The MCP server SHALL communicate using JSON-RPC 2.0 over a transport layer.

#### Scenario: Accept JSON-RPC requests
- **WHEN** a client sends a valid JSON-RPC request
- **THEN** the server SHALL parse and process the request

#### Scenario: Return JSON-RPC responses
- **WHEN** processing completes
- **THEN** the server SHALL return a JSON-RPC 2.0 response object

### Requirement: Transport layer
The MCP server SHALL support multiple transport mechanisms.

#### Scenario: TCP transport
- **WHEN** running on any platform
- **THEN** it SHALL accept connections on localhost:7777 by default

#### Scenario: Unix socket transport
- **WHEN** running on Linux or Mac
- **THEN** it SHALL support Unix domain sockets at `.flashgrep/mcp.sock`

### Requirement: query method
The MCP server SHALL expose a query method for text search.

#### Scenario: Query with text parameter
- **WHEN** the query method is called with text parameter
- **THEN** it SHALL return search results matching the text

#### Scenario: Query with limit parameter
- **WHEN** the query method includes a limit parameter
- **THEN** it SHALL return at most that many results

### Requirement: get_slice method
The MCP server SHALL expose a method to retrieve file slices.

#### Scenario: Get specific line range
- **WHEN** get_slice is called with file_path, start_line, end_line
- **THEN** it SHALL return the exact content of those lines

#### Scenario: Handle missing file
- **WHEN** the requested file does not exist
- **THEN** it SHALL return an appropriate error response

### Requirement: get_symbol method
The MCP server SHALL expose a method to find symbols by name.

#### Scenario: Find symbol by name
- **WHEN** get_symbol is called with symbol_name
- **THEN** it SHALL return all occurrences of that symbol

### Requirement: list_files method
The MCP server SHALL expose a method to list indexed files.

#### Scenario: List all files
- **WHEN** list_files is called
- **THEN** it SHALL return a list of all indexed file paths

### Requirement: stats method
The MCP server SHALL expose a method to retrieve index statistics.

#### Scenario: Get index stats
- **WHEN** stats is called
- **THEN** it SHALL return: total files, total chunks, index size, last update time

### Requirement: Minimal responses
The MCP server SHALL return compact, structured responses.

#### Scenario: No full file content by default
- **WHEN** returning query results
- **THEN** it SHALL NOT include full file content unless explicitly requested via get_slice
