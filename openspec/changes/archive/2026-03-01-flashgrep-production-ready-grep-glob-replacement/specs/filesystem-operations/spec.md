## ADDED Requirements

### Requirement: Cross-platform filesystem lifecycle operations
The CLI MUST provide file and directory lifecycle operations for create, list, stat, and remove actions that behave consistently across Windows, macOS, and Linux for equivalent inputs.

#### Scenario: Create files and directories
- **WHEN** a user runs a filesystem create command for a missing file or directory path
- **THEN** the CLI MUST create the target and return a deterministic success response

#### Scenario: Remove files and directories with clear mode semantics
- **WHEN** a user removes a file, empty directory, or recursive directory target with explicit mode flags
- **THEN** the CLI MUST apply the requested mode deterministically and report errors when mode and target type conflict

### Requirement: Deterministic copy and move behavior
The CLI MUST provide copy and move operations for files and directories with explicit overwrite behavior and deterministic outcomes across supported platforms.

#### Scenario: Copy without overwrite permission
- **WHEN** a destination already exists and overwrite is not enabled
- **THEN** the operation MUST fail with a deterministic conflict error and MUST NOT modify destination content

#### Scenario: Move with overwrite enabled
- **WHEN** overwrite is explicitly enabled for a move operation
- **THEN** the CLI MUST replace the destination according to documented semantics and preserve deterministic success or failure signaling

### Requirement: Automation-safe destructive operation controls
Mutating filesystem operations that can delete or overwrite data MUST support non-interactive safety controls including explicit force behavior and dry-run preview mode.

#### Scenario: Dry-run for destructive command
- **WHEN** a user runs a destructive filesystem operation with dry-run enabled
- **THEN** the CLI MUST report intended actions without changing the filesystem

#### Scenario: Force mode in non-interactive execution
- **WHEN** a user runs a destructive operation in automation with force enabled
- **THEN** the CLI MUST execute without interactive prompts and return deterministic machine-consumable status

### Requirement: Structured filesystem metadata output
Filesystem list and stat operations MUST provide stable machine-readable output fields including path, type, size, and modification metadata suitable for scripting.

#### Scenario: Query metadata for a path
- **WHEN** a user requests stat output for an existing file or directory
- **THEN** the response MUST include required metadata fields with stable field names

#### Scenario: List directory contents deterministically
- **WHEN** a user lists directory contents with sorting and limit controls
- **THEN** the CLI MUST return entries in deterministic order and enforce the requested bounds
