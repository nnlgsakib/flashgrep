## ADDED Requirements

### Requirement: Query command for indexed text search
The CLI SHALL provide an indexed text search command for grep-like workflows.

#### Scenario: Run text query
- **WHEN** a user runs the query command with search text
- **THEN** the command SHALL return ranked matches from the existing index
- **AND** each match SHALL include file path and line range

#### Scenario: Limit query output
- **WHEN** a user specifies a result limit
- **THEN** the command SHALL return at most N matches

### Requirement: Files command for index-aware listing
The CLI SHALL provide an index-backed file listing command for glob-like workflows.

#### Scenario: List indexed files
- **WHEN** a user runs the files command
- **THEN** it SHALL return paths from the current repository index

#### Scenario: Filter listed files
- **WHEN** a user provides a filename/path filter
- **THEN** the command SHALL return only matching indexed files

### Requirement: Symbol command for structural lookup
The CLI SHALL provide symbol lookup for fast structural navigation.

#### Scenario: Find symbol occurrences
- **WHEN** a user runs the symbol command with a symbol name
- **THEN** it SHALL return matching symbol definitions/usages from index metadata

### Requirement: Slice command for targeted code extraction
The CLI SHALL provide code slicing by file and line range.

#### Scenario: Extract line range
- **WHEN** a user runs the slice command with file path and line bounds
- **THEN** it SHALL return the requested line range from that file

### Requirement: Script-friendly output mode
The CLI search commands SHALL support structured output for automation.

#### Scenario: JSON output
- **WHEN** a user requests structured output mode
- **THEN** the command SHALL emit valid JSON for machine parsing
