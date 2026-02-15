## ADDED Requirements

### Requirement: File system monitoring
The file watcher SHALL monitor the repository for file changes.

#### Scenario: Watch repository root
- **WHEN** the daemon starts
- **THEN** it SHALL begin watching the repository root directory recursively

#### Scenario: Handle file creation
- **WHEN** a new file is created in the repository
- **THEN** it SHALL index the new file within 100ms

#### Scenario: Handle file modification
- **WHEN** an indexed file is modified
- **THEN** it SHALL re-index the file within 100ms

#### Scenario: Handle file deletion
- **WHEN** an indexed file is deleted
- **THEN** it SHALL remove the file from the index within 100ms

### Requirement: Debounced updates
The file watcher SHALL batch rapid changes to avoid excessive re-indexing.

#### Scenario: Multiple rapid changes
- **WHEN** a file changes multiple times within 500ms
- **THEN** it SHALL perform a single re-index after the burst completes

### Requirement: Graceful error handling
The file watcher SHALL handle errors without crashing the daemon.

#### Scenario: Permission denied
- **WHEN** the watcher encounters a permission error
- **THEN** it SHALL log the error and continue watching other files

#### Scenario: File moved during indexing
- **WHEN** a file is moved or deleted during re-indexing
- **THEN** it SHALL handle the error gracefully and skip the file

### Requirement: Concurrent project watchers
The watcher subsystem SHALL support concurrent watchers for multiple project roots.

#### Scenario: Multiple active project roots
- **WHEN** watchers are started for different repository roots
- **THEN** file events SHALL be processed independently per repository
- **AND** indexing for one repository SHALL NOT block watcher operation for another repository

### Requirement: Project-scoped watcher registry
The watcher subsystem SHALL maintain project-scoped runtime state for lifecycle operations.

#### Scenario: Path canonicalization for identity
- **WHEN** a watcher is started or stopped
- **THEN** the project path SHALL be canonicalized before lookup
- **AND** equivalent paths SHALL map to a single watcher identity

#### Scenario: Stale watcher state cleanup
- **WHEN** watcher state exists but process is no longer alive
- **THEN** the subsystem SHALL treat it as stale and clean it up
- **AND** the next start command SHALL launch a fresh watcher

### Requirement: Background launch reliability
The watcher subsystem SHALL provide clear failure semantics for background launches.

#### Scenario: Background process launch failure
- **WHEN** background watcher spawn fails
- **THEN** the CLI SHALL return a non-zero exit code
- **AND** it SHALL print a clear error message with failure reason

#### Scenario: Successful background launch
- **WHEN** background watcher spawn succeeds
- **THEN** it SHALL record enough metadata to support status and stop commands for that project

### Requirement: Ignore file change reconciliation
The watcher SHALL react to `.flashgrepignore` updates by reconciling indexed content.

#### Scenario: Ignore file modified
- **WHEN** `.flashgrepignore` is created or modified
- **THEN** watcher SHALL reload ignore patterns
- **AND** it SHALL trigger reconciliation for already indexed files

#### Scenario: Newly ignored paths are pruned
- **WHEN** reconciliation finds indexed files now matching ignore rules
- **THEN** watcher/indexer SHALL remove those files from index and metadata
- **AND** subsequent searches SHALL NOT return those files

### Requirement: Ignore reconciliation observability
The watcher SHALL provide user-visible diagnostics for ignore reconciliation.

#### Scenario: Reconciliation summary
- **WHEN** ignore reconciliation completes
- **THEN** it SHALL log counts for files removed and files kept

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
