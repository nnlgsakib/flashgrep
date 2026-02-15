## ADDED Requirements

### Requirement: Bootstrap includes skill-based policies
The bootstrap response SHALL include skill-based tool injection metadata.

#### Scenario: Skill metadata in bootstrap response
- **WHEN** bootstrap is called
- **THEN** the response SHALL include skill document reference or content
- **AND** it SHALL indicate that skill-based guidance is available

### Requirement: Tool priority from skill context
The bootstrap SHALL derive tool priority policies from skill definitions.

#### Scenario: Skill-derived priorities
- **WHEN** bootstrap response includes skill metadata
- **THEN** it SHALL reflect the tool priorities defined in the skill
- **AND** it SHALL maintain consistency with explicit policy metadata

## MODIFIED Requirements

### Requirement: Flashgrep-first tool policy in bootstrap response
The bootstrap response MUST include explicit operational guidance and machine-readable policy metadata that prioritizes Flashgrep-native tools over generic filesystem and native agent tools for applicable tasks.

#### Scenario: Agent receives strict policy guidance
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include guidance to prefer `query`, `files`, `symbol`, `read_code`, and `write_code` before generic grep/glob/native tool fallbacks

#### Scenario: Policy metadata is present for client enforcement
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include structured policy fields for preferred tool order, fallback gates, and enforcement strength

#### Scenario: Skill-based policy enhancement
- **WHEN** bootstrap succeeds and skill is installed
- **THEN** the response MUST include skill-based context for tool selection
- **AND** it SHALL provide examples from the skill definition

### Requirement: Idempotent session bootstrap behavior
The system MUST support idempotent bootstrap semantics within a running server session while preserving policy metadata consistency.

#### Scenario: First bootstrap call in session
- **WHEN** bootstrap is called for the first time in a server session
- **THEN** the response MUST indicate `injected` status and include policy metadata

#### Scenario: Repeated bootstrap call in same session
- **WHEN** bootstrap is called again in the same server session without force refresh
- **THEN** the response MUST indicate `already_injected` status and preserve equivalent policy semantics

#### Scenario: Skill metadata consistency
- **WHEN** bootstrap is called multiple times
- **THEN** skill-based guidance SHALL remain consistent across calls
- **AND** it SHALL reflect the currently installed skill version
