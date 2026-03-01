## MODIFIED Requirements

### Requirement: Index command
The CLI SHALL provide an index command for initial repository indexing and SHALL prompt for neural model download when required assets are missing.

#### Scenario: Run index command
- **WHEN** the user runs `flashgrep index`
- **THEN** it SHALL scan and index the current directory

#### Scenario: Index specific directory
- **WHEN** the user runs `flashgrep index /path/to/repo`
- **THEN** it SHALL index the specified directory

#### Scenario: Show indexing progress
- **WHEN** the index command runs
- **THEN** it SHALL display real-time progress to stdout

#### Scenario: Prompt for missing neural model on index start
- **WHEN** model assets are missing at index startup
- **THEN** the CLI SHALL prompt user to download `BAAI/bge-small-en-v1.5`
- **AND** if the user declines, it SHALL continue indexing normally without download

#### Scenario: Success exit code
- **WHEN** indexing completes successfully
- **THEN** it SHALL exit with code 0

#### Scenario: Error exit code
- **WHEN** indexing encounters a fatal error
- **THEN** it SHALL exit with a non-zero code and print the error

### Requirement: Start command
The CLI SHALL provide a start command to run the daemon and SHALL handle model prompt behavior before initial indexing begins.

#### Scenario: Run start command
- **WHEN** the user runs `flashgrep start`
- **THEN** it SHALL start the file watcher and MCP server

#### Scenario: Prompt for missing neural model on start
- **WHEN** watcher startup begins initial indexing and model assets are missing
- **THEN** the CLI SHALL prompt user to download `BAAI/bge-small-en-v1.5`
- **AND** if the user declines, daemon startup SHALL continue with normal non-neural indexing behavior

#### Scenario: Print server address
- **WHEN** the daemon starts
- **THEN** it SHALL print the server address to stdout

#### Scenario: Run continuously
- **WHEN** the daemon is running
- **THEN** it SHALL continue running until interrupted

#### Scenario: Graceful shutdown
- **WHEN** the user sends SIGINT or SIGTERM
- **THEN** it SHALL shut down gracefully, closing all connections and saving state
