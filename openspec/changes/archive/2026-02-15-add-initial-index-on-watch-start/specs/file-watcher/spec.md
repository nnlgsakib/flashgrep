## ADDED Requirements

### Requirement: Initial indexing on watcher start
The file watcher SHALL perform an initial scan and index of all files when it starts.

#### Scenario: Scan repository on startup
- **WHEN** the file watcher is started for a repository
- **THEN** it SHALL begin scanning all files in the repository
- **AND** it SHALL build an index of current file state

#### Scenario: Progress logging for large repositories
- **WHEN** scanning a repository with more than 1000 files
- **THEN** it SHALL log progress periodically (e.g., every 1000 files)
- **AND** it SHALL indicate that initial indexing is in progress

### Requirement: Non-blocking initial indexing
The file watcher SHALL start monitoring for real-time changes immediately without waiting for initial indexing to complete.

#### Scenario: Concurrent watching and indexing
- **WHEN** the file watcher starts
- **THEN** it SHALL immediately begin watching for file system events
- **AND** it SHALL simultaneously perform initial indexing in the background
- **AND** real-time file changes SHALL be detected during indexing

#### Scenario: File changed during indexing
- **WHEN** a file is modified while initial indexing is still in progress
- **THEN** the file system watcher SHALL detect the change
- **AND** the file SHALL be re-indexed normally

### Requirement: Change detection from persisted state
The file watcher SHALL compare current repository state with the previously persisted index to detect changes that occurred while offline.

#### Scenario: Detect files added while offline
- **WHEN** initial indexing finds a file
- **AND** that file was not in the persisted index
- **THEN** it SHALL emit a synthetic "file created" event

#### Scenario: Detect files modified while offline
- **WHEN** initial indexing finds a file that exists in persisted index
- **AND** the file's modification time or content hash differs from persisted state
- **THEN** it SHALL emit a synthetic "file modified" event

#### Scenario: Detect files deleted while offline
- **WHEN** initial indexing completes
- **AND** a file exists in persisted index but was not found during scan
- **THEN** it SHALL emit a synthetic "file deleted" event

### Requirement: Persisted index state
The file watcher SHALL maintain a persisted index of file metadata for comparison between sessions.

#### Scenario: Persist index after scan
- **WHEN** initial indexing completes for a file
- **THEN** it SHALL update the persisted index with file metadata (path, size, mtime, hash)

#### Scenario: Load previous index on startup
- **WHEN** the file watcher starts
- **THEN** it SHALL load the persisted index from previous session
- **AND** use it as baseline for change detection

### Requirement: Respect ignore patterns during initial scan
The file watcher SHALL apply .flashgrepignore patterns during initial indexing.

#### Scenario: Skip ignored files during scan
- **WHEN** performing initial indexing
- **AND** a file matches .flashgrepignore patterns
- **THEN** it SHALL skip indexing that file
- **AND** it SHALL NOT emit events for ignored files

#### Scenario: Handle ignore file changes
- **WHEN** .flashgrepignore is modified between sessions
- **THEN** the initial scan SHALL use the current ignore patterns
- **AND** files now ignored SHALL be removed from index (via existing reconciliation)
