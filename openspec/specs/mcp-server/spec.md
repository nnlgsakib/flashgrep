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

### Requirement: Skill bootstrap MCP method
The MCP server MUST expose a bootstrap method that returns agent skill injection content and status metadata.

#### Scenario: Bootstrap method returns structured success response
- **WHEN** a valid bootstrap request is received
- **THEN** the server MUST return a structured response containing bootstrap status, canonical trigger, and skill payload metadata

#### Scenario: Bootstrap method validates trigger
- **WHEN** a bootstrap request uses an unsupported trigger value
- **THEN** the server MUST return a structured error response with code `invalid_trigger`

### Requirement: Bootstrap compatibility with existing MCP methods
The MCP server MUST preserve behavior of existing methods while adding bootstrap support.

#### Scenario: Existing tools remain callable after bootstrap support is added
- **WHEN** clients call existing methods such as `query`, `get_slice`, `get_symbol`, `list_files`, `stats`, `read_code`, and `write_code`
- **THEN** the server MUST continue to process them according to existing contracts

### Requirement: Bootstrap error handling
The MCP server MUST provide machine-readable bootstrap error responses for skill loading failures.

#### Scenario: Skill file load failure returns typed error
- **WHEN** the server cannot load `skills/SKILL.md` during bootstrap
- **THEN** the server MUST return an error with a typed code (for example `skill_not_found` or `skill_unreadable`) and remediation details

### Requirement: Cross-transport method consistency
Equivalent MCP methods exposed through different transport handlers MUST remain behaviorally consistent.

#### Scenario: Bootstrap aliases are handled consistently
- **WHEN** bootstrap is requested using any supported alias (`bootstrap_skill`, `flashgrep-init`, `flashgrep_init`, `fgrep-boot`, `fgrep_boot`) via either stdio or TCP MCP paths
- **THEN** the server MUST normalize to the same canonical trigger and return equivalent bootstrap semantics

#### Scenario: Existing core methods keep consistent behavior
- **WHEN** clients call shared methods (`query`, `get_slice`, `get_symbol`, `list_files`, `stats`, `read_code`, `write_code`) through different MCP transport handlers
- **THEN** the server MUST preserve method-level behavior and required response fields across transports

### Requirement: Centralized shared handler usage for duplicated MCP logic
The MCP implementation MUST use shared internal handlers for duplicated logic paths instead of maintaining divergent transport-specific copies.

#### Scenario: Bootstrap logic is centralized
- **WHEN** bootstrap processing is updated (trigger validation, skill loading, idempotency, metadata)
- **THEN** both transport handlers MUST consume the same shared logic path so behavior updates apply consistently

#### Scenario: Tool definition aliases remain synchronized
- **WHEN** tool aliases or bootstrap-related definitions are added or modified
- **THEN** the MCP tool listing and invocation paths MUST reflect the same alias set without transport-specific drift

### Requirement: Glob MCP contract supports advanced options
The MCP server MUST expose an expanded glob tool contract supporting advanced filtering, traversal, ordering, and limit controls.

#### Scenario: Advanced glob options are accepted
- **WHEN** a client calls glob with advanced options such as `extensions`, `exclude`, `max_depth`, `include_hidden`, `sort_by`, or `limit`
- **THEN** the server MUST validate and apply those options and return matching results

#### Scenario: Invalid option combinations return structured errors
- **WHEN** a client sends unsupported or incompatible glob option combinations
- **THEN** the server MUST return a structured parameter error without partial ambiguous behavior

### Requirement: Glob performance remains suitable for large repositories
The MCP server MUST apply traversal-time filtering and short-circuit strategies that reduce unnecessary scanning overhead.

#### Scenario: Early pruning with excludes and depth
- **WHEN** exclude filters or depth bounds are provided
- **THEN** the server MUST prune traversal early instead of scanning and post-filtering full trees
