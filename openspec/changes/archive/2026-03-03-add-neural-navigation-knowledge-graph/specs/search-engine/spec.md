## MODIFIED Requirements

### Requirement: Full-text search
The search engine SHALL perform lexical full-text queries against indexed repository content and SHALL provide an optional neural-assisted retrieval mode that uses local index and knowledge-graph candidates as the primary retrieval source.

#### Scenario: Basic text query
- **WHEN** a user submits a text query
- **THEN** the system SHALL return matching chunks ranked by relevance

#### Scenario: Limit results
- **WHEN** a query specifies a limit parameter
- **THEN** the system SHALL return at most N results

#### Scenario: Neural-assisted query uses graph candidates first
- **WHEN** neural query mode is enabled and a natural-language query is submitted
- **THEN** the system SHALL retrieve bounded candidates from lexical/semantic index and knowledge-graph neighborhoods before provider inference

#### Scenario: Provider failure falls back deterministically
- **WHEN** neural-assisted query mode is selected and provider inference fails or times out
- **THEN** the system SHALL return deterministic lexical/index-backed results with typed diagnostics
