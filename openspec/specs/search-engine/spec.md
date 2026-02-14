## ADDED Requirements

### Requirement: Full-text search
The search engine SHALL perform full-text queries against the indexed content.

#### Scenario: Basic text query
- **WHEN** a user submits a text query
- **THEN** the system SHALL return matching chunks ranked by relevance

#### Scenario: Limit results
- **WHEN** a query specifies a limit parameter
- **THEN** the system SHALL return at most N results

### Requirement: Relevance ranking
The search engine SHALL rank results using multiple factors.

#### Scenario: Symbol match boost
- **WHEN** a query matches a detected symbol name
- **THEN** those results SHALL receive a relevance boost

#### Scenario: Proximity ranking
- **WHEN** multiple search terms are provided
- **THEN** results with terms closer together SHALL rank higher

#### Scenario: Recency ranking
- **WHEN** files have different modification times
- **THEN** more recently modified files SHALL receive a slight boost

#### Scenario: File depth penalty
- **WHEN** matching files are at different directory depths
- **THEN** shallower files SHALL rank slightly higher than deep nested files

### Requirement: Query response time
The search engine SHALL respond within performance constraints.

#### Scenario: Medium repository query
- **WHEN** querying a medium-sized repository
- **THEN** the response SHALL complete in under 50ms

### Requirement: Result format
The search engine SHALL return structured results with metadata.

#### Scenario: Result includes location
- **WHEN** returning search results
- **THEN** each result SHALL include: file_path, start_line, end_line

#### Scenario: Result includes symbol
- **WHEN** a result contains a detected symbol
- **THEN** it SHALL include the symbol_name field

#### Scenario: Result includes score
- **WHEN** returning search results
- **THEN** each result SHALL include a relevance_score

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
