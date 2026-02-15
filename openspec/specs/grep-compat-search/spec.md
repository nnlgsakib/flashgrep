## ADDED Requirements

### Requirement: Indexed grep-compatible text matching
The system MUST provide grep-compatible text matching over indexed repository content, including literal and regex matching modes with deterministic result limits.

#### Scenario: Regex search over indexed content
- **WHEN** a caller submits a regex pattern and search scope
- **THEN** the system MUST return matches that satisfy the regex using indexed content without full-tree rescans

#### Scenario: Literal search mode
- **WHEN** a caller requests literal matching mode
- **THEN** the system MUST treat special regex characters as plain text and return literal matches only

### Requirement: Case and path scoping controls
The system MUST support case-sensitive/case-insensitive search behavior and path-based include or exclude scoping for grep-compatible workflows.

#### Scenario: Case-insensitive grep workflow
- **WHEN** a caller enables case-insensitive search
- **THEN** matches MUST include content regardless of letter case differences

#### Scenario: Scoped search by include and exclude paths
- **WHEN** a caller provides include and exclude path constraints
- **THEN** results MUST be limited to included paths and MUST omit excluded paths

### Requirement: Context-aware deterministic output
The system MUST support grep-style context output behavior with deterministic ordering and bounded response sizes for automation.

#### Scenario: Return matching lines with surrounding context
- **WHEN** a caller requests context lines around each match
- **THEN** the response MUST include matching lines and requested adjacent context lines with stable formatting

#### Scenario: Enforce deterministic capped output
- **WHEN** result or byte limits are configured
- **THEN** the system MUST enforce those limits deterministically and return metadata indicating truncation when applicable
