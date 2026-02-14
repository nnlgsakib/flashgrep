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

### Requirement: Search command discoverability in help
The CLI help SHALL expose new fast search commands and usage.

#### Scenario: Top-level help includes search commands
- **WHEN** a user runs `flashgrep --help`
- **THEN** help output SHALL list all new search-oriented commands

#### Scenario: Command help includes examples
- **WHEN** a user runs help for a search command
- **THEN** it SHALL show concise examples for common grep/glob replacement workflows

### Requirement: Consistent error messaging for search commands
The CLI SHALL provide actionable errors for missing index, invalid paths, and malformed arguments.

#### Scenario: Missing index
- **WHEN** a search command is run before indexing
- **THEN** it SHALL return a clear error instructing the user to run `flashgrep index`

#### Scenario: Invalid command arguments
- **WHEN** a user provides invalid required arguments
- **THEN** the CLI SHALL return a non-zero exit and usage guidance
