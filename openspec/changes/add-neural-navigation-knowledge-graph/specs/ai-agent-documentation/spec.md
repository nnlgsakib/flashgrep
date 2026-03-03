## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
Agent guidance MUST be optimized for low-token operation while enforcing strict Flashgrep-first tool routing with deterministic fallback gates, documentation MUST define embedded-bootstrap behavior as the default source of policy, and `skills/SKILL.md` MUST direct agents to index-first retrieval. Neural-assisted navigation MAY be used only when enabled, and model prompts MUST be bounded to candidate evidence.

#### Scenario: Skill guidance describes embedded init source
- **WHEN** users or agents review skill/bootstrap guidance
- **THEN** documentation MUST state that canonical policy guidance is injected from embedded payload at initialization

#### Scenario: Skill doc enforces index-first retrieval ordering
- **WHEN** an agent chooses a search approach for discovery tasks
- **THEN** the guidance MUST direct the agent to lexical/programmatic index-backed retrieval first
- **AND** it MUST define allowed fallback reasons before non-Flashgrep-native search is used

#### Scenario: Skill doc documents optional neural mode constraints
- **WHEN** neural-assisted search is documented
- **THEN** docs MUST require explicit enablement, bounded candidate context, and token-efficiency safeguards

#### Scenario: Skill doc includes compliance troubleshooting guidance
- **WHEN** an agent deviates from policy-compliant search order
- **THEN** documentation MUST include concise remediation steps and metadata fields to inspect for restoring compliant routing

#### Scenario: Skill guidance routes filesystem actions to MCP tools
- **WHEN** an agent needs file or directory create/read/write/list/stat/copy/move/remove actions
- **THEN** the skill guidance MUST direct the agent to Flashgrep MCP filesystem tools first

#### Scenario: Skill guidance preserves fallback gates
- **WHEN** native fallback tools are considered
- **THEN** the skill guidance MUST require explicit fallback gate and reason code metadata before fallback is allowed
