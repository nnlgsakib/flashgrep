## ADDED Requirements

### Requirement: Index command
The CLI SHALL provide an index command for initial repository indexing.

#### Scenario: Run index command
- **WHEN** the user runs `flashgrep index`
- **THEN** it SHALL scan and index the current directory

#### Scenario: Index specific directory
- **WHEN** the user runs `flashgrep index /path/to/repo`
- **THEN** it SHALL index the specified directory

#### Scenario: Show indexing progress
- **WHEN** the index command runs
- **THEN** it SHALL display real-time progress to stdout

#### Scenario: Success exit code
- **WHEN** indexing completes successfully
- **THEN** it SHALL exit with code 0

#### Scenario: Error exit code
- **WHEN** indexing encounters a fatal error
- **THEN** it SHALL exit with a non-zero code and print the error

### Requirement: Start command
The CLI SHALL provide a start command to run the daemon.

#### Scenario: Run start command
- **WHEN** the user runs `flashgrep start`
- **THEN** it SHALL start the file watcher and MCP server

#### Scenario: Print server address
- **WHEN** the daemon starts
- **THEN** it SHALL print the server address to stdout

#### Scenario: Run continuously
- **WHEN** the daemon is running
- **THEN** it SHALL continue running until interrupted

#### Scenario: Graceful shutdown
- **WHEN** the user sends SIGINT or SIGTERM
- **THEN** it SHALL shut down gracefully, closing all connections and saving state

### Requirement: Help and version
The CLI SHALL provide standard help and version commands.

#### Scenario: Show help
- **WHEN** the user runs `flashgrep --help` or `flashgrep -h`
- **THEN** it SHALL display usage information for all commands

#### Scenario: Show version
- **WHEN** the user runs `flashgrep --version` or `flashgrep -V`
- **THEN** it SHALL display the version number

### Requirement: Flashgrepignore documentation
The CLI documentation SHALL mention the `.flashgrepignore` file support.

#### Scenario: Help mentions ignore file
- **WHEN** the user views help documentation
- **THEN** it SHALL describe the `.flashgrepignore` file and its format

#### Scenario: Ignore file location
- **WHEN** describing the ignore functionality
- **THEN** it SHALL specify that `.flashgrepignore` is read from the repository root
