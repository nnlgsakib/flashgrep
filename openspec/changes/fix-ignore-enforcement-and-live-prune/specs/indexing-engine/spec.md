## ADDED Requirements

### Requirement: Strict ignore enforcement during indexing
The indexing engine SHALL enforce `.flashgrepignore` patterns for all scan and index paths.

#### Scenario: Ignored assistant directories are skipped
- **WHEN** `.flashgrepignore` includes `.opencode/` (or similar ignored directory patterns)
- **THEN** files under those directories SHALL NOT be indexed
- **AND** they SHALL NOT appear in query, files, or symbol results

#### Scenario: Ignore matching uses normalized repo-relative paths
- **WHEN** the indexer evaluates ignore patterns
- **THEN** it SHALL match against normalized repository-relative paths
- **AND** matching behavior SHALL be consistent across platforms and path separators

### Requirement: Ignore-aware incremental indexing
The indexing engine SHALL reject indexing operations for paths that are currently ignored.

#### Scenario: File event targets ignored path
- **WHEN** an incremental update receives a file path that matches ignore patterns
- **THEN** the engine SHALL skip indexing that file
- **AND** it SHALL log the skip reason at debug/info level
