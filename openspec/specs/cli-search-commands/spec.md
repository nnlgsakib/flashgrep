## MODIFIED Requirements

### Requirement: Query command for indexed text search
The CLI SHALL provide indexed lexical search commands, SHALL support continuation-aware execution for arbitrarily large logical result sets, and SHALL expose compatibility options required for production script replacement.

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

### Requirement: Files command for index-aware listing
The CLI SHALL provide an index-backed file listing command for glob-like workflows, SHALL support deterministic continuation windows for very large match sets, and SHALL expose filter controls needed for production glob replacement.

#### Scenario: List indexed files
- **WHEN** a user runs the files command
- **THEN** it SHALL return paths from the current repository index

#### Scenario: Filter listed files
- **WHEN** a user provides a filename/path filter
- **THEN** the command SHALL return only matching indexed files

#### Scenario: Complete large file set retrieval
- **WHEN** matching paths exceed one output page
- **THEN** the CLI SHALL provide continuation/pagination controls to retrieve all matching paths without gaps or duplicates

#### Scenario: Apply glob replacement controls
- **WHEN** a user provides recursive, hidden-path, include, and exclude controls
- **THEN** the files command SHALL apply those controls consistently and deterministically across supported platforms

## ADDED Requirements

### Requirement: CLI filesystem command group
The CLI SHALL provide a filesystem command group for create, list, stat, copy, move, and remove operations with non-interactive automation-safe controls.

#### Scenario: Execute filesystem operations from CLI
- **WHEN** a user invokes a filesystem subcommand with valid arguments
- **THEN** the command SHALL execute the operation and return deterministic output and exit status

#### Scenario: Use dry-run and force controls for mutations
- **WHEN** a user invokes mutating filesystem subcommands with dry-run or force options
- **THEN** the CLI SHALL honor those controls exactly as documented without interactive ambiguity
