## ADDED Requirements

### Requirement: Canonical skill payload is embedded in binary
The bootstrap system MUST include a canonical skill payload embedded at build time and MUST serve this payload without requiring repository-local files.

#### Scenario: Init succeeds without filesystem skill file
- **WHEN** bootstrap/init is invoked in an environment where `skills/SKILL.md` is missing or unreadable
- **THEN** the response MUST still include the canonical skill payload sourced from embedded content

#### Scenario: Embedded payload source is explicit
- **WHEN** bootstrap/init returns skill content
- **THEN** metadata MUST include a payload source field indicating `embedded`

### Requirement: Embedded bootstrap executes at initialization
The system MUST inject policy guidance during initialization so clients receive Flashgrep-first routing policy before subsequent tool usage.

#### Scenario: First init call injects policy
- **WHEN** a client performs first bootstrap/init call in a session
- **THEN** the response MUST indicate policy/skill injection completed for that session

#### Scenario: Repeated init remains idempotent
- **WHEN** the same session calls bootstrap/init again without force refresh
- **THEN** the response MUST indicate already-injected semantics while preserving equivalent policy metadata

### Requirement: Optional repo override is explicitly gated
If repository-local skill override is supported, it MUST be opt-in and MUST NOT be required for successful bootstrap.

#### Scenario: Override disabled uses embedded payload
- **WHEN** override is not explicitly enabled
- **THEN** bootstrap MUST use embedded payload regardless of repository file state

#### Scenario: Override enabled reports source and fallback
- **WHEN** override is enabled and repo skill file fails validation or read
- **THEN** bootstrap MUST fall back to embedded payload and report typed fallback reason in metadata
