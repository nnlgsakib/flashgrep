## ADDED Requirements

### Requirement: Watch single file for changes
The system SHALL provide a command to watch a single file for changes.

#### Scenario: Watch file modifications
- **WHEN** user executes fs-watch with path="/config.json" and events=["modify"]
- **THEN** system starts watching the file
- **AND** returns a watch identifier
- **AND** notifies when file content changes

#### Scenario: Watch file creation
- **WHEN** user executes fs-watch with path="/temp.txt" and events=["create"]
- **AND** the file does not exist initially
- **THEN** system starts watching for file creation
- **AND** notifies when file is created

#### Scenario: Watch file deletion
- **WHEN** user executes fs-watch with path="/important.txt" and events=["delete"]
- **THEN** system starts watching the file
- **AND** notifies when file is deleted

### Requirement: Watch directory for changes
The system SHALL support watching entire directories.

#### Scenario: Watch directory non-recursively
- **WHEN** user executes fs-watch with path="/logs" and recursive=false
- **THEN** system watches for changes in /logs directory only
- **AND** does not watch subdirectories

#### Scenario: Watch directory recursively
- **WHEN** user executes fs-watch with path="/project" and recursive=true
- **THEN** system watches for changes in /project and all subdirectories
- **AND** notifies for any file changes in the tree

#### Scenario: Watch with file pattern filter
- **WHEN** user executes fs-watch with path="/src", recursive=true, and pattern="*.ts"
- **THEN** system only notifies for changes to .ts files
- **AND** ignores changes to other file types

### Requirement: Debounce change notifications
The system SHALL debounce rapid successive changes to prevent notification flooding.

#### Scenario: Debounce rapid changes
- **WHEN** user executes fs-watch with path="/file.txt" and debounceMs=300
- **AND** file is modified multiple times within 300ms
- **THEN** system emits only one notification after debounce period
- **AND** notification contains latest state

#### Scenario: No debounce option
- **WHEN** user executes fs-watch with path="/file.txt" and debounceMs=0
- **AND** file is modified
- **THEN** system emits notification immediately for each change

### Requirement: Stop watching
The system SHALL provide a command to stop watching.

#### Scenario: Stop specific watch
- **WHEN** user executes fs-watch with action="stop" and watchId="watch-123"
- **THEN** system stops the specified watch
- **AND** releases associated resources
- **AND** no further notifications for that watch

#### Scenario: Stop all watches
- **WHEN** user executes fs-watch with action="stop-all"
- **THEN** system stops all active watches
- **AND** releases all watch resources

### Requirement: Return structured watch events
The system SHALL provide detailed information in watch notifications.

#### Scenario: File change event
- **WHEN** fs-watch emits a notification for file modification
- **THEN** event includes: watchId, path, eventType, timestamp
- **AND** optionally includes new file content if configured

#### Scenario: Batch change events
- **WHEN** multiple files change within debounce window
- **AND** batch=true is configured
- **THEN** system emits single event with array of changes
- **AND** each change includes path and eventType
