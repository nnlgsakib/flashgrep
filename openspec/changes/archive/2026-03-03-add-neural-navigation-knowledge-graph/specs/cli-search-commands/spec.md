## MODIFIED Requirements

### Requirement: Query command for indexed text search
The CLI SHALL provide indexed lexical search commands, SHALL support optional neural navigation mode for natural-language discovery, SHALL support continuation-aware execution for arbitrarily large logical result sets, and SHALL expose compatibility options required for production script replacement.

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

#### Scenario: Use grep-compatibility flags
- **WHEN** a user provides grep-compatibility options for case, context, or literal/fixed matching
- **THEN** the query command SHALL apply those options with deterministic behavior and documented exit statuses

#### Scenario: Run neural navigation query
- **WHEN** a user runs query in neural mode with natural-language intent text
- **THEN** the command SHALL use neural-assisted index-first retrieval and return ranked file and line matches

#### Scenario: Neural mode requested but disabled
- **WHEN** a user requests neural mode and neural navigation is not enabled/configured
- **THEN** the command SHALL return a deterministic configuration guidance error
