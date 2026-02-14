## Purpose

Define requirements for native MCP bootstrap that injects Flashgrep skill guidance into agent sessions.

## ADDED Requirements

### Requirement: Native skill bootstrap trigger
The system MUST provide a native MCP bootstrap method that accepts recognized init triggers and returns Flashgrep skill guidance for agent startup.

#### Scenario: Bootstrap via flashgrep-init
- **WHEN** a client calls bootstrap with trigger `flashgrep-init`
- **THEN** the system MUST return a successful bootstrap response containing injected skill guidance

#### Scenario: Bootstrap via fgrep-boot alias
- **WHEN** a client calls bootstrap with trigger `fgrep-boot`
- **THEN** the system MUST treat it as a valid alias and return the same bootstrap semantics as `flashgrep-init`

### Requirement: Skill content source and payload metadata
The system MUST source bootstrap content from the repository skill file and include metadata that allows agents to reason about versioning and caching.

#### Scenario: Skill file is available
- **WHEN** bootstrap is requested and `skills/SKILL.md` exists and is readable
- **THEN** the response MUST include skill content and metadata fields including source path and skill hash/version identifiers

#### Scenario: Skill file is unavailable
- **WHEN** bootstrap is requested and `skills/SKILL.md` is missing or unreadable
- **THEN** the response MUST return a structured error code with recovery guidance

### Requirement: Idempotent session bootstrap behavior
The system MUST support idempotent bootstrap semantics within a running server session.

#### Scenario: First bootstrap call in session
- **WHEN** bootstrap is called for the first time in a server session
- **THEN** the response MUST indicate `injected` status

#### Scenario: Repeated bootstrap call in same session
- **WHEN** bootstrap is called again in the same server session without force refresh
- **THEN** the response MUST indicate `already_injected` status and avoid duplicating unnecessary payload content

### Requirement: Flashgrep-first tool policy in bootstrap response
The bootstrap response MUST include explicit operational guidance that prioritizes Flashgrep-native tools over generic filesystem search tools for applicable tasks.

#### Scenario: Agent receives policy guidance
- **WHEN** bootstrap succeeds
- **THEN** the response MUST include guidance to prefer `query`, `files`, `symbol`, `read_code`, and `write_code` before generic grep/glob style fallbacks
