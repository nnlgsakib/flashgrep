## ADDED Requirements

### Requirement: Read single file with caching
The system SHALL provide a command to read a single file with automatic caching for the session duration.

#### Scenario: Read uncached file
- **WHEN** user executes fs-read with filePath="/path/to/file.txt"
- **THEN** system reads file from disk and returns its contents
- **AND** system caches the content for subsequent reads

#### Scenario: Read cached file
- **WHEN** user executes fs-read with filePath="/path/to/file.txt" that was read earlier in the session
- **AND** the file has not been modified since last read
- **THEN** system returns cached content without disk access

#### Scenario: Cache invalidation on modification
- **WHEN** user executes fs-read with filePath="/path/to/file.txt"
- **AND** the file has been modified since last cached read
- **THEN** system reads file from disk and updates cache
- **AND** returns new content

### Requirement: Read multiple files in batch
The system SHALL support reading multiple files in a single command.

#### Scenario: Batch read multiple files
- **WHEN** user executes fs-read with filePaths=["/file1.txt", "/file2.txt"]
- **THEN** system reads all specified files
- **AND** returns a map of file paths to their contents
- **AND** applies caching to each file individually

#### Scenario: Batch read with missing files
- **WHEN** user executes fs-read with filePaths=["/exists.txt", "/missing.txt"]
- **AND** one or more files do not exist
- **THEN** system returns contents for existing files
- **AND** includes error information for missing files
- **AND** operation does not fail entirely

### Requirement: Support file encoding options
The system SHALL support different text encodings for file reading.

#### Scenario: Read with UTF-8 encoding
- **WHEN** user executes fs-read with filePath="/file.txt" and encoding="utf-8"
- **THEN** system decodes file content as UTF-8 text

#### Scenario: Read with binary mode
- **WHEN** user executes fs-read with filePath="/file.bin" and encoding="binary"
- **THEN** system returns raw binary data (base64 encoded for transport)

#### Scenario: Read with custom encoding
- **WHEN** user executes fs-read with filePath="/file.txt" and encoding="latin1"
- **THEN** system decodes file content using the specified encoding
