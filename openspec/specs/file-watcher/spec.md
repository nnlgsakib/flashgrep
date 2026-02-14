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
