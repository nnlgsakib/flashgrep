## MODIFIED Requirements

### Requirement: Query command for indexed text search
The CLI SHALL provide an indexed text search command for grep-like workflows with production-ready parity options for regex mode, literal mode, case handling, path scoping, and context output.

#### Scenario: Run text query
- **WHEN** a user runs the query command with search text
- **THEN** the command SHALL return ranked matches from the existing index
- **AND** each match SHALL include file path and line range

#### Scenario: Regex and literal modes are selectable
- **WHEN** a user explicitly selects regex or literal matching mode
- **THEN** the command SHALL apply the selected mode consistently for all matched output

#### Scenario: Limit query output
- **WHEN** a user specifies a result limit
- **THEN** the command SHALL return at most N matches

#### Scenario: Context lines are requested
- **WHEN** a user requests context lines around matches
- **THEN** the command SHALL include matching lines with the requested before/after context

### Requirement: Files command for index-aware listing
The CLI SHALL provide an index-backed file listing command for glob-like workflows with deterministic sorting and advanced filtering controls.

#### Scenario: List indexed files
- **WHEN** a user runs the files command
- **THEN** it SHALL return paths from the current repository index

#### Scenario: Filter listed files
- **WHEN** a user provides a filename/path filter
- **THEN** the command SHALL return only matching indexed files

#### Scenario: Deterministic sorted output
- **WHEN** a user provides sort and order controls
- **THEN** the command SHALL return results in deterministic order suitable for scripting
