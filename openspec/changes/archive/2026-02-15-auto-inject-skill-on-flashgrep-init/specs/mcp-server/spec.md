## ADDED Requirements

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
