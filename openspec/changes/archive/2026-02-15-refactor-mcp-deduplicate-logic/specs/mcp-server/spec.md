## ADDED Requirements

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
