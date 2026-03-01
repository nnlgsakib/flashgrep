## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
`skills/SKILL.md` MUST remain optimized for low-token operation while enforcing strict Flashgrep-first routing, and MUST direct agents to MCP filesystem tools for file lifecycle operations before native host read/write/glob fallbacks.

#### Scenario: Skill guidance routes filesystem actions to MCP tools
- **WHEN** an agent needs file or directory create/read/write/list/stat/copy/move/remove actions
- **THEN** the skill guidance MUST direct the agent to Flashgrep MCP filesystem tools first

#### Scenario: Skill guidance preserves fallback gates
- **WHEN** native fallback tools are considered
- **THEN** the skill guidance MUST require explicit fallback gate and reason code metadata before fallback is allowed

### Requirement: README documents MCP filesystem operations
The documentation set MUST define MCP filesystem tool usage and missing-path diagnostics so agents can branch deterministically without native tool assumptions.

#### Scenario: README includes MCP filesystem mappings
- **WHEN** users review MCP usage documentation
- **THEN** README MUST include examples and parameter guidance for MCP filesystem lifecycle tools

#### Scenario: README includes not-found diagnostics contract
- **WHEN** users troubleshoot missing files or directories
- **THEN** README MUST document typed not-found fields and reason codes returned by MCP tools
