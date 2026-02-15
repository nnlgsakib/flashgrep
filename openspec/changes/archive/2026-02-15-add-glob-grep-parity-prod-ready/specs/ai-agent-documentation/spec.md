## ADDED Requirements

### Requirement: README documents grep/glob replacement workflows
The documentation set MUST provide explicit migration guidance in `README.md` for replacing common grep and glob workflows with Flashgrep CLI and MCP operations.

#### Scenario: README includes migration mappings
- **WHEN** users review CLI and MCP usage guidance
- **THEN** the README MUST provide concrete grep/glob-to-Flashgrep workflow mappings and production usage examples

#### Scenario: README defines production expectations
- **WHEN** users evaluate Flashgrep as a replacement
- **THEN** the README MUST explain determinism, bounded outputs, and index freshness expectations

### Requirement: Skill guidance is token-efficient and action-oriented
`skills/SKILL.md` MUST be optimized for low-token agent operation while preserving correct Flashgrep-first tool selection and fallback guidance.

#### Scenario: Skill doc provides compact decision flow
- **WHEN** an agent loads the skill file
- **THEN** the skill MUST prioritize compact decision rules over verbose narrative and still preserve operational correctness

#### Scenario: Skill doc preserves primary tool ordering
- **WHEN** an agent chooses search or read/write tools
- **THEN** the skill MUST direct the agent to Flashgrep-native tools first before generic alternatives
