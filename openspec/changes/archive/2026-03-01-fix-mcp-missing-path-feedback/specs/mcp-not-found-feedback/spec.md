## ADDED Requirements

### Requirement: Typed not-found diagnostics for MCP path operations
The MCP layer MUST return explicit machine-readable not-found diagnostics whenever a path-targeting tool receives a file or directory path that does not exist.

#### Scenario: Missing file returns typed not-found payload
- **WHEN** a caller invokes a file-reading MCP tool with a non-existent file path
- **THEN** the response MUST mark the call as error and include typed fields identifying `error = not_found`, the missing `target_path`, and `target_kind = file`

#### Scenario: Missing directory returns typed not-found payload
- **WHEN** a caller invokes a directory-targeting MCP tool with a non-existent directory path
- **THEN** the response MUST mark the call as error and include typed fields identifying `error = not_found`, the missing `target_path`, and `target_kind = directory`

### Requirement: Deterministic reason codes for not-found cases
Not-found diagnostics MUST include stable reason codes so agents can branch behavior deterministically across tools and transports.

#### Scenario: Not-found reason code is stable
- **WHEN** a not-found condition is returned from any supported path-aware MCP tool
- **THEN** the payload MUST include a deterministic `reason_code` indicating path absence semantics

#### Scenario: Transport consistency for reason codes
- **WHEN** equivalent missing-path requests are executed over stdio and TCP handlers
- **THEN** the returned `error` and `reason_code` semantics MUST be equivalent across transports
