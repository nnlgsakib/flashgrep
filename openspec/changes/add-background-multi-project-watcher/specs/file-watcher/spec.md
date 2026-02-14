## ADDED Requirements

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
