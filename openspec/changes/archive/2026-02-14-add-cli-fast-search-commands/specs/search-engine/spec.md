## ADDED Requirements

### Requirement: CLI result shaping
The search engine SHALL provide CLI-friendly result shaping controls.

#### Scenario: Deterministic result ordering
- **WHEN** the same query is run repeatedly on unchanged index data
- **THEN** result ordering SHALL remain stable for a fixed limit

#### Scenario: Bounded response size
- **WHEN** a command specifies a limit
- **THEN** the search engine SHALL enforce the limit before CLI rendering

### Requirement: Fast path for repeated local queries
The search engine SHALL optimize repeated CLI queries using existing index structures.

#### Scenario: Repeated query performance
- **WHEN** users run repeated text queries on an existing index
- **THEN** query latency SHALL remain within the existing performance envelope of indexed search

### Requirement: Output metadata completeness
The search engine SHALL return required metadata for CLI formatting modes.

#### Scenario: Plain text mode fields
- **WHEN** CLI renders plain text output
- **THEN** each result SHALL include file path, line range, and score/context fields needed for concise display

#### Scenario: JSON mode fields
- **WHEN** CLI renders JSON output
- **THEN** each result SHALL include consistent, schema-like field names across commands
