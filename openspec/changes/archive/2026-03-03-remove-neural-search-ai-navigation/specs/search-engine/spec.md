## MODIFIED Requirements

### Requirement: Full-text search
The search engine SHALL perform lexical full-text queries against indexed repository content.

#### Scenario: Basic text query
- **WHEN** a user submits a text query
- **THEN** the system SHALL return matching chunks ranked by relevance

#### Scenario: Limit results
- **WHEN** a query specifies a limit parameter
- **THEN** the system SHALL return at most N results
