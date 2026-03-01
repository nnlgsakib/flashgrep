## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
Agent guidance MUST be optimized for low-token operation while enforcing strict Flashgrep-first tool routing with deterministic fallback gates, documentation MUST define embedded-bootstrap behavior as the default source of policy, and `skills/SKILL.md` MUST direct agents to MCP filesystem tools for file lifecycle operations before native host read/write/glob fallbacks.

#### Scenario: Skill guidance describes embedded init source
- **WHEN** users or agents review skill/bootstrap guidance
- **THEN** documentation MUST state that canonical policy guidance is injected from embedded payload at initialization

#### Scenario: Skill doc enforces primary tool ordering
- **WHEN** an agent chooses search or read/write tools
- **THEN** the guidance MUST direct the agent to Flashgrep-native tools first and require explicit gated reasons before native fallback tools are used

#### Scenario: Skill doc includes compliance troubleshooting guidance
- **WHEN** an agent deviates from Flashgrep-first behavior
- **THEN** documentation MUST include concise remediation steps and metadata fields to inspect for restoring policy-compliant routing

#### Scenario: Skill guidance routes filesystem actions to MCP tools
- **WHEN** an agent needs file or directory create/read/write/list/stat/copy/move/remove actions
- **THEN** the skill guidance MUST direct the agent to Flashgrep MCP filesystem tools first

#### Scenario: Skill guidance preserves fallback gates
- **WHEN** native fallback tools are considered
- **THEN** the skill guidance MUST require explicit fallback gate and reason code metadata before fallback is allowed

### Requirement: README documents grep/glob replacement workflows
The documentation set MUST provide explicit migration guidance in `README.md` for replacing common grep and glob workflows with Flashgrep CLI and MCP operations, including embedded-bootstrap policy guarantees and policy-compliant fallback behavior.

#### Scenario: README includes migration mappings
- **WHEN** users review CLI and MCP usage guidance
- **THEN** the README MUST provide concrete grep/glob-to-Flashgrep workflow mappings and production usage examples

#### Scenario: README defines production expectations
- **WHEN** users evaluate Flashgrep as a replacement
- **THEN** the README MUST explain determinism, bounded outputs, index freshness expectations, embedded policy injection expectations, and fallback gates

#### Scenario: README includes embedded bootstrap troubleshooting
- **WHEN** users diagnose policy injection issues
- **THEN** the README MUST describe how to verify payload source and bootstrap state metadata

### Requirement: README documents MCP filesystem operations
The documentation set MUST define MCP filesystem tool usage and missing-path diagnostics so agents can branch deterministically without native tool assumptions.

#### Scenario: README includes MCP filesystem mappings
- **WHEN** users review MCP usage documentation
- **THEN** README MUST include examples and parameter guidance for MCP filesystem lifecycle tools

#### Scenario: README includes not-found diagnostics contract
- **WHEN** users troubleshoot missing files or directories
- **THEN** README MUST document typed not-found fields and reason codes returned by MCP tools
