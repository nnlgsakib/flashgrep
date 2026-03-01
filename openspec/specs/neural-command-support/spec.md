## ADDED Requirements

### Requirement: Embedded model bootstrap and cache lifecycle
The system SHALL download and cache `BAAI/bge-small-en-v1.5` under `.flashgrep/` on first neural operation and SHALL reuse cached assets on subsequent runs.

#### Scenario: First neural operation downloads model
- **WHEN** a user invokes a neural query or neural indexing path and model assets are not present
- **THEN** the system SHALL download required model artifacts into `.flashgrep/model-cache/`
- **AND** it SHALL persist metadata needed to validate cache completeness for future runs

#### Scenario: Cached model is reused
- **WHEN** a user invokes a neural operation and valid model assets already exist in `.flashgrep/model-cache/`
- **THEN** the system SHALL use the local cache without re-downloading

#### Scenario: Download failure returns actionable error
- **WHEN** model download fails due to network or integrity issues
- **THEN** the system SHALL fail the neural operation with a deterministic error message that identifies recovery steps

### Requirement: Vector-backed semantic retrieval
The system SHALL support semantic retrieval over project-indexed vectors and SHALL return code navigation-friendly matches.

#### Scenario: Semantic query returns relevant project locations
- **WHEN** a user submits a natural-language query such as finding authentication code
- **THEN** the system SHALL return ranked results from indexed vectors
- **AND** each result SHALL include repository-relative file path and line range

#### Scenario: Query uses project-local vector knowledge
- **WHEN** semantic retrieval executes
- **THEN** the system SHALL search vectors generated from indexed project chunks
- **AND** it SHALL not require external knowledge sources for repository matches

### Requirement: Deterministic neural output contract
Neural query output SHALL remain deterministic and automation-safe for unchanged index/model state.

#### Scenario: Stable ordering for unchanged input
- **WHEN** the same neural query runs repeatedly against unchanged vector/index state
- **THEN** the returned top-N result ordering SHALL remain stable

#### Scenario: Structured result fields for tooling
- **WHEN** neural results are emitted in JSON mode
- **THEN** each result SHALL include consistent fields for file path, line range, and relevance score
