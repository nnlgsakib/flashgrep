## ADDED Requirements

### Requirement: README MCP stdio setup documentation
The system SHALL document a complete MCP stdio setup flow for Flashgrep in `README.md`.

#### Scenario: Full MCP client config example
- **WHEN** a user needs to configure an MCP client for Flashgrep
- **THEN** the README SHALL include a full JSON configuration block with a `flashgrep` entry
- **AND** the config SHALL use command `flashgrep mcp-stdio`
- **AND** the config SHALL show `enabled: true`
- **AND** the config SHALL include `RUST_LOG: info` in environment settings

#### Scenario: Setup steps include validation
- **WHEN** a user follows README MCP setup instructions
- **THEN** the steps SHALL include indexing the repository before MCP usage
- **AND** the steps SHALL include launching stdio mode
- **AND** the steps SHALL include a simple validation outcome for successful connection

### Requirement: README skill discovery guidance
The system SHALL document agent-agnostic skill file locations and usage guidance.

#### Scenario: Generic skill path documented
- **WHEN** a user looks for a skill that works with any coding agent
- **THEN** the README SHALL reference `skills/SKILL.md`
- **AND** the README SHALL describe this as the primary, agent-agnostic skill path

#### Scenario: Optional OpenCode-managed path documented
- **WHEN** a user uses OpenCode-managed skills
- **THEN** the README SHALL reference `.opencode/skills/flashgrep-mcp/SKILL.md`
- **AND** the README SHALL describe this path as optional and OpenCode-specific

#### Scenario: Cross-linking from MCP API section
- **WHEN** a user reads the MCP API methods section
- **THEN** the README SHALL link to the setup and skill discovery section to reduce duplication
