## MODIFIED Requirements

### Requirement: Query command for indexed text search
The CLI SHALL provide an indexed text search command for grep-like workflows and SHALL support continuation-aware execution for arbitrarily large logical result sets.

#### Scenario: Run text query
- **WHEN** a user runs the query command with search text
- **THEN** the command SHALL return ranked matches from the existing index
- **AND** each match SHALL include file path and line range

#### Scenario: Limit query output
- **WHEN** a user specifies a result limit
- **THEN** the command SHALL return at most N matches

#### Scenario: Complete large query via continuation mode
- **WHEN** a query spans more matches/content than one output window
- **THEN** the CLI SHALL support deterministic continuation until full logical completion

### Requirement: Files command for index-aware listing
The CLI SHALL provide an index-backed file listing command for glob-like workflows and SHALL support deterministic continuation windows for very large match sets.

#### Scenario: List indexed files
- **WHEN** a user runs the files command
- **THEN** it SHALL return paths from the current repository index

#### Scenario: Filter listed files
- **WHEN** a user provides a filename/path filter
- **THEN** the command SHALL return only matching indexed files

#### Scenario: Complete large file set retrieval
- **WHEN** matching paths exceed one output page
- **THEN** the CLI SHALL provide continuation/pagination controls to retrieve all matching paths without gaps or duplicates
