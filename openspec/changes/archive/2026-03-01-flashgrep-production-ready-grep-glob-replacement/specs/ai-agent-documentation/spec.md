## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
`skills/SKILL.md` MUST be optimized for low-token agent operation while enforcing strict Flashgrep-first tool routing, deterministic fallback gates, and explicit guidance for grep/glob replacement plus filesystem operation usage.

#### Scenario: Skill doc provides compact decision flow
- **WHEN** an agent loads the skill file
- **THEN** the skill MUST prioritize compact decision rules over verbose narrative and preserve operational correctness

#### Scenario: Skill doc enforces primary tool ordering
- **WHEN** an agent chooses search or read/write tools
- **THEN** the skill MUST direct the agent to Flashgrep-native tools first and require explicit gated reasons before native fallback tools are used

#### Scenario: Skill doc includes filesystem operation routing
- **WHEN** an agent needs file or directory lifecycle actions
- **THEN** the skill MUST direct it to the documented Flashgrep filesystem workflow with platform-safe defaults

### Requirement: README documents grep/glob replacement workflows
The documentation set MUST provide explicit migration guidance in `README.md` for replacing common grep and glob workflows with Flashgrep CLI and MCP operations, including filesystem operation mapping and policy-compliant fallback behavior.

#### Scenario: README includes migration mappings
- **WHEN** users review CLI and MCP usage guidance
- **THEN** the README MUST provide concrete grep/glob-to-Flashgrep workflow mappings and production usage examples

#### Scenario: README defines production expectations
- **WHEN** users evaluate Flashgrep as a replacement
- **THEN** the README MUST explain determinism, bounded outputs, index freshness expectations, safety controls for filesystem mutations, and policy fallback gates

#### Scenario: README documents cross-platform behavior
- **WHEN** users run workflows on Windows, macOS, and Linux
- **THEN** the README MUST describe platform-specific path and filesystem behavior guarantees for automation
