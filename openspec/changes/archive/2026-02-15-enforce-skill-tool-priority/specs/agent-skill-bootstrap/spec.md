## MODIFIED Requirements

### Requirement: Flashgrep-first tool policy in bootstrap response
The bootstrap response MUST include explicit operational guidance and machine-readable policy metadata that prioritizes Flashgrep-native tools over generic filesystem and native agent tools for applicable tasks.

#### Scenario: Agent receives strict policy guidance
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include guidance to prefer `query`, `files`, `symbol`, `read_code`, and `write_code` before generic grep/glob/native tool fallbacks

#### Scenario: Policy metadata is present for client enforcement
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include structured policy fields for preferred tool order, fallback gates, and enforcement strength

### Requirement: Idempotent session bootstrap behavior
The system MUST support idempotent bootstrap semantics within a running server session while preserving policy metadata consistency.

#### Scenario: First bootstrap call in session
- **WHEN** bootstrap is called for the first time in a server session
- **THEN** the response MUST indicate `injected` status and include policy metadata

#### Scenario: Repeated bootstrap call in same session
- **WHEN** bootstrap is called again in the same server session without force refresh
- **THEN** the response MUST indicate `already_injected` status and preserve equivalent policy semantics
