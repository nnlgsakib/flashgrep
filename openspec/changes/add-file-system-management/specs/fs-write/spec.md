## ADDED Requirements

### Requirement: Write single file
The system SHALL provide a command to write content to a single file.

#### Scenario: Create new file
- **WHEN** user executes fs-write with filePath="/path/to/new.txt" and content="Hello World"
- **THEN** system creates the file with specified content
- **AND** returns success confirmation

#### Scenario: Overwrite existing file
- **WHEN** user executes fs-write with filePath="/path/to/exists.txt" and content="New content"
- **AND** the file already exists
- **THEN** system overwrites the file with new content
- **AND** invalidates any cached version of the file

#### Scenario: Create with parent directories
- **WHEN** user executes fs-write with filePath="/new/dir/file.txt" and content="data"
- **AND** parent directories do not exist
- **THEN** system creates all necessary parent directories
- **AND** creates the file with specified content

### Requirement: Write multiple files in batch
The system SHALL support writing multiple files in a single command.

#### Scenario: Batch write multiple files
- **WHEN** user executes fs-write with writes=[{"filePath":"/a.txt","content":"A"},{"filePath":"/b.txt","content":"B"}]
- **THEN** system creates all specified files with their contents
- **AND** returns results for each file operation
- **AND** invalidates cache for any overwritten files

#### Scenario: Batch write with partial failures
- **WHEN** user executes fs-write with multiple files
- **AND** one file cannot be written (e.g., permission denied)
- **THEN** system attempts all writes
- **AND** returns individual success/failure for each file
- **AND** does not roll back successful writes

### Requirement: Support write options
The system SHALL support various write options for flexibility.

#### Scenario: Write with append mode
- **WHEN** user executes fs-write with filePath="/log.txt", content="Line 2", and mode="append"
- **THEN** system appends content to existing file
- **AND** creates file if it doesn't exist

#### Scenario: Write with atomic option
- **WHEN** user executes fs-write with filePath="/important.txt", content="data", and atomic=true
- **THEN** system writes to temporary file first
- **AND** renames temporary file to target atomically
- **AND** ensures no partial writes on failure

#### Scenario: Skip if exists
- **WHEN** user executes fs-write with filePath="/exists.txt", content="data", and overwrite=false
- **AND** the file already exists
- **THEN** system does not modify the file
- **AND** returns indication that file was skipped
