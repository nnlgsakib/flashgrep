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
