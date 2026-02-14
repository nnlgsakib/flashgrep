## ADDED Requirements

### Requirement: Background start mode
The CLI SHALL support launching the file watcher in background mode.

#### Scenario: Start watcher in background
- **WHEN** the user runs `flashgrep start -b`
- **THEN** it SHALL start watcher execution as a detached process
- **AND** it SHALL return control to the terminal immediately
- **AND** it SHALL print confirmation including the target repository path

#### Scenario: Preserve foreground default behavior
- **WHEN** the user runs `flashgrep start` without `-b`
- **THEN** it SHALL continue running in the foreground until interrupted

### Requirement: Multi-project watcher lifecycle commands
The CLI SHALL manage watcher lifecycle independently per project root.

#### Scenario: Start watchers for different repositories
- **WHEN** the user runs start for two different repository paths
- **THEN** it SHALL run watchers for both repositories concurrently
- **AND** each watcher SHALL be tracked using its canonical project path

#### Scenario: Duplicate start for same repository
- **WHEN** a watcher is already active for a repository and user starts it again
- **THEN** the CLI SHALL not start a duplicate watcher
- **AND** it SHALL print a clear "already running" message

#### Scenario: Stop by project path
- **WHEN** the user runs stop for a specific repository path
- **THEN** the CLI SHALL stop only that repository watcher
- **AND** it SHALL leave other project watchers running
