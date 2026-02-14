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
