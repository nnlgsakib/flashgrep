## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
`skills/SKILL.md` MUST be optimized for low-token agent operation while enforcing strict Flashgrep-first tool routing with deterministic fallback gates.

#### Scenario: Skill doc provides compact decision flow
- **WHEN** an agent loads the skill file
- **THEN** the skill MUST prioritize compact decision rules over verbose narrative and preserve operational correctness

#### Scenario: Skill doc enforces primary tool ordering
- **WHEN** an agent chooses search or read/write tools
- **THEN** the skill MUST direct the agent to Flashgrep-native tools first and require explicit gated reasons before native fallback tools are used

#### Scenario: Skill doc includes compliance troubleshooting guidance
- **WHEN** an agent deviates from Flashgrep-first behavior
- **THEN** the skill MUST include concise remediation steps for restoring policy-compliant tool routing

### Requirement: README documents grep/glob replacement workflows
The documentation set MUST provide explicit migration guidance in `README.md` for replacing common grep and glob workflows with Flashgrep CLI and MCP operations, including policy-compliant fallback behavior.

#### Scenario: README includes migration mappings
- **WHEN** users review CLI and MCP usage guidance
- **THEN** the README MUST provide concrete grep/glob-to-Flashgrep workflow mappings and production usage examples

#### Scenario: README defines production expectations
- **WHEN** users evaluate Flashgrep as a replacement
- **THEN** the README MUST explain determinism, bounded outputs, index freshness expectations, and policy fallback gates
