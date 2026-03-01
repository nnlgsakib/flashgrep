## MODIFIED Requirements

### Requirement: Flashgrep-first tool policy in bootstrap response
The bootstrap response MUST include explicit operational guidance and machine-readable policy metadata that prioritizes Flashgrep-native tools over generic filesystem and native agent tools for applicable tasks, and this guidance MUST be available from embedded payloads during initialization.

#### Scenario: Agent receives strict policy guidance
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include guidance to prefer `query`, `files`, `symbol`, `read_code`, and `write_code` before generic grep/glob/native tool fallbacks

#### Scenario: Policy metadata is present for client enforcement
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include structured policy fields for preferred tool order, fallback gates, and enforcement strength

#### Scenario: Policy is available without repository skill file
- **WHEN** bootstrap runs and repository `skills/SKILL.md` is unavailable
- **THEN** equivalent policy guidance and metadata MUST still be returned from embedded content

### Requirement: Idempotent session bootstrap behavior
The system MUST support idempotent bootstrap semantics within a running server session while preserving policy metadata consistency and payload-source transparency.

#### Scenario: First bootstrap call in session
- **WHEN** bootstrap is called for the first time in a server session
- **THEN** the response MUST indicate `injected` status and include policy metadata

#### Scenario: Repeated bootstrap call in same session
- **WHEN** bootstrap is called again in the same server session without force refresh
- **THEN** the response MUST indicate `already_injected` status and preserve equivalent policy semantics

#### Scenario: Bootstrap reports payload source deterministically
- **WHEN** bootstrap returns policy guidance
- **THEN** the response MUST include deterministic payload-source metadata indicating whether content came from embedded payload or explicit override
