## ADDED Requirements

### Requirement: Copy files
The system SHALL provide a command to copy files from source to destination.

#### Scenario: Copy single file
- **WHEN** user executes fs-batch with operation="copy", source="/src/file.txt", destination="/dst/file.txt"
- **THEN** system copies the file to destination
- **AND** preserves original file

#### Scenario: Copy with overwrite
- **WHEN** user executes fs-batch with operation="copy" and destination file exists
- **AND** overwrite option is true
- **THEN** system overwrites the destination file

#### Scenario: Copy without overwrite
- **WHEN** user executes fs-batch with operation="copy" and destination file exists
- **AND** overwrite option is false (default)
- **THEN** system does not copy and reports file exists

#### Scenario: Copy directory recursively
- **WHEN** user executes fs-batch with operation="copy", source="/src/dir", destination="/dst/dir", and recursive=true
- **THEN** system copies entire directory tree
- **AND** preserves directory structure

### Requirement: Move/rename files
The system SHALL provide a command to move or rename files.

#### Scenario: Move single file
- **WHEN** user executes fs-batch with operation="move", source="/old/path.txt", destination="/new/path.txt"
- **THEN** system moves the file to new location
- **AND** file no longer exists at source path

#### Scenario: Rename file
- **WHEN** user executes fs-batch with operation="move" where source and destination are in same directory
- **THEN** system renames the file

#### Scenario: Move directory recursively
- **WHEN** user executes fs-batch with operation="move", source="/old/dir", destination="/new/dir", and recursive=true
- **THEN** system moves entire directory tree
- **AND** preserves all contents and structure

### Requirement: Delete files and directories
The system SHALL provide a command to delete files and directories.

#### Scenario: Delete single file
- **WHEN** user executes fs-batch with operation="delete" and path="/file.txt"
- **THEN** system deletes the file
- **AND** invalidates any cached content for the file

#### Scenario: Delete directory
- **WHEN** user executes fs-batch with operation="delete", path="/dir", and recursive=false
- **AND** directory is empty
- **THEN** system deletes the empty directory

#### Scenario: Delete directory recursively
- **WHEN** user executes fs-batch with operation="delete", path="/dir", and recursive=true
- **AND** directory contains files and subdirectories
- **THEN** system deletes the directory and all contents

### Requirement: Execute batch operations atomically
The system SHALL support atomic execution of multiple operations.

#### Scenario: Atomic batch execution
- **WHEN** user executes fs-batch with operations=[...] and atomic=true
- **THEN** system validates all operations can succeed
- **AND** executes all operations
- **AND** if any operation fails, all changes are rolled back

#### Scenario: Non-atomic batch execution
- **WHEN** user executes fs-batch with operations=[...] and atomic=false (default)
- **THEN** system executes operations sequentially
- **AND** individual failures do not affect other operations
- **AND** partial results are retained
