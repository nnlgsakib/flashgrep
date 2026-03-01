## MODIFIED Requirements

### Requirement: Skill guidance is token-efficient and action-oriented
Agent guidance MUST be optimized for low-token operation while enforcing strict Flashgrep-first tool routing with deterministic fallback gates, and documentation MUST define embedded-bootstrap behavior as the default source of policy.

#### Scenario: Skill guidance describes embedded init source
- **WHEN** users or agents review skill/bootstrap guidance
- **THEN** documentation MUST state that canonical policy guidance is injected from embedded payload at initialization

#### Scenario: Skill doc enforces primary tool ordering
- **WHEN** an agent chooses search or read/write tools
- **THEN** the guidance MUST direct the agent to Flashgrep-native tools first and require explicit gated reasons before native fallback tools are used

#### Scenario: Skill doc includes compliance troubleshooting guidance
- **WHEN** an agent deviates from Flashgrep-first behavior
- **THEN** documentation MUST include concise remediation steps and metadata fields to inspect for restoring policy-compliant routing

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
