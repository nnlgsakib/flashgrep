## MODIFIED Requirements

### Requirement: Full-text search
The search engine SHALL perform lexical full-text queries and semantic vector queries against indexed repository content.

#### Scenario: Basic text query
- **WHEN** a user submits a text query
- **THEN** the system SHALL return matching chunks ranked by relevance

#### Scenario: Semantic text query
- **WHEN** a user submits a natural-language query in semantic mode
- **THEN** the system SHALL return semantically matched chunks ranked by relevance

#### Scenario: Hybrid text query
- **WHEN** a user submits a query in hybrid mode
- **THEN** the system SHALL combine lexical and semantic scoring deterministically

#### Scenario: Limit results
- **WHEN** a query specifies a limit parameter
- **THEN** the system SHALL return at most N results

### Requirement: Result format
The search engine SHALL return structured results with metadata for lexical and neural ranking workflows.

#### Scenario: Result includes location
- **WHEN** returning search results
- **THEN** each result SHALL include: file_path, start_line, end_line

#### Scenario: Result includes symbol
- **WHEN** a result contains a detected symbol
- **THEN** it SHALL include the symbol_name field

#### Scenario: Result includes score
- **WHEN** returning search results
- **THEN** each result SHALL include a relevance_score
