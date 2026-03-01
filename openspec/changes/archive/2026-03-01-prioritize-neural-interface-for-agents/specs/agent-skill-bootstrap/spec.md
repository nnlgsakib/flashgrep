## MODIFIED Requirements

### Requirement: Flashgrep-first tool policy in bootstrap response
The bootstrap response MUST include explicit operational guidance and machine-readable policy metadata that prioritizes Flashgrep-native tools over generic filesystem and native agent tools for applicable tasks, and this guidance MUST be available from embedded payloads during initialization. For search workflows, guidance and metadata MUST instruct neural interface usage as primary and programmatic retrieval as second priority fallback.

#### Scenario: Agent receives strict policy guidance
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include guidance to prefer `query`, `files`, `symbol`, `read_code`, and `write_code` before generic grep/glob/native tool fallbacks

#### Scenario: Bootstrap guidance sets neural-first search behavior
- **WHEN** bootstrap policy is injected for an agent session
- **THEN** guidance MUST instruct agents to run neural/semantic query behavior first for discovery requests
- **AND** guidance MUST define programmatic query usage as second-choice fallback

#### Scenario: Policy metadata is present for client enforcement
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include structured policy fields for preferred tool order, fallback gates, and enforcement strength

#### Scenario: Policy is available without repository skill file
- **WHEN** bootstrap runs and repository `skills/SKILL.md` is unavailable
- **THEN** equivalent policy guidance and metadata MUST still be returned from embedded content
