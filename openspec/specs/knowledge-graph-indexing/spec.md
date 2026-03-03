## ADDED Requirements

### Requirement: Knowledge graph generation during indexing
The indexing pipeline SHALL generate and persist a repository knowledge graph for neural navigation when neural mode is enabled.

#### Scenario: Build graph on enabled index run
- **WHEN** the user enables neural navigation and runs indexing
- **THEN** the system SHALL build graph nodes and edges from files, symbols, and references

#### Scenario: Skip graph generation when disabled
- **WHEN** neural navigation is disabled
- **THEN** the system SHALL skip graph artifact generation and complete lexical indexing normally

### Requirement: Incremental graph updates
The system SHALL update knowledge-graph artifacts incrementally as files change.

#### Scenario: File update triggers incremental graph refresh
- **WHEN** changed files are detected by indexing or watcher updates
- **THEN** the system SHALL update only affected graph nodes and edges

#### Scenario: Deterministic graph revision tracking
- **WHEN** graph artifacts are updated
- **THEN** the system SHALL record a deterministic revision marker usable for cache keys and query consistency
