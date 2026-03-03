## ADDED Requirements

### Requirement: Optional neural navigation query mode
The system SHALL provide an optional neural navigation mode for natural-language repository discovery while preserving lexical navigation as the default behavior.

#### Scenario: User runs natural-language navigation query
- **WHEN** a user submits a query such as "find code that sorts names" in neural mode
- **THEN** the system SHALL return ranked repository locations with file path and line range

#### Scenario: Neural mode is disabled
- **WHEN** neural mode is not enabled in configuration
- **THEN** the system SHALL return a deterministic guidance error and SHALL preserve lexical query behavior

### Requirement: Index-first neural retrieval
Neural navigation SHALL use local indexed candidates as primary evidence, and model calls SHALL only operate on bounded candidate context.

#### Scenario: Candidate retrieval before model inference
- **WHEN** neural navigation executes
- **THEN** candidate chunks and symbols SHALL be selected from local index and knowledge-graph neighborhoods before any provider call

#### Scenario: Model receives bounded context
- **WHEN** a model request is made for neural navigation
- **THEN** the request SHALL include only bounded candidate snippets and metadata, not full repository content
