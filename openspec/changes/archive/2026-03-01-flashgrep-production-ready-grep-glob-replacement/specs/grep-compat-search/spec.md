## MODIFIED Requirements

### Requirement: Indexed grep-compatible text matching
The system MUST provide grep-compatible text matching over indexed repository content, including literal, fixed-string, and regex matching modes, while preserving deterministic behavior for large result sets.

#### Scenario: Regex search over indexed content
- **WHEN** a caller submits a regex pattern and search scope
- **THEN** the system MUST return matches that satisfy the regex using indexed content without full-tree rescans

#### Scenario: Literal search mode
- **WHEN** a caller requests literal matching mode
- **THEN** the system MUST treat special regex characters as plain text and return literal matches only

#### Scenario: Fixed-string multi-pattern matching
- **WHEN** a caller provides one or more fixed-string patterns for grep-style matching
- **THEN** the system MUST match any provided fixed-string pattern without regex interpretation

### Requirement: Case and path scoping controls
The system MUST support case-sensitive/case-insensitive search behavior and path-based include or exclude scoping for grep-compatible workflows, including deterministic handling of platform path separators.

#### Scenario: Case-insensitive grep workflow
- **WHEN** a caller enables case-insensitive search
- **THEN** matches MUST include content regardless of letter case differences

#### Scenario: Scoped search by include and exclude paths
- **WHEN** a caller provides include and exclude path constraints
- **THEN** results MUST be limited to included paths and MUST omit excluded paths

#### Scenario: Cross-platform path normalization in scope filters
- **WHEN** a caller provides path scopes using platform-native separators
- **THEN** scope filtering MUST resolve paths deterministically and consistently across supported platforms

### Requirement: Context-aware deterministic output
The system MUST support grep-style context output behavior with deterministic ordering, bounded response sizes, and script-safe output formatting for automation.

#### Scenario: Return matching lines with surrounding context
- **WHEN** a caller requests context lines around each match
- **THEN** the response MUST include matching lines and requested adjacent context lines with stable formatting

#### Scenario: Enforce deterministic capped output
- **WHEN** result or byte limits are configured
- **THEN** the system MUST enforce those limits deterministically and return metadata indicating truncation when applicable

#### Scenario: Script-safe output fields
- **WHEN** a caller requests machine-oriented output mode
- **THEN** the system MUST emit stable field structure for file, line, match text, and context metadata

## ADDED Requirements

### Requirement: Grep-compatible exit status and error channel behavior
Search commands MUST provide deterministic grep-compatible exit status semantics and must separate match output from operational errors for script reliability.

#### Scenario: Match found exit status
- **WHEN** at least one match is found and no fatal execution error occurs
- **THEN** the command MUST return success exit status

#### Scenario: No match exit status
- **WHEN** the search completes successfully but finds no matches
- **THEN** the command MUST return the documented no-match exit status distinct from execution failure

#### Scenario: Operational failure exit status
- **WHEN** the command encounters an unrecoverable error
- **THEN** the command MUST return a failure exit status and emit diagnostics on the error channel
