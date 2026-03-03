## MODIFIED Requirements

### Requirement: Index command
The CLI SHALL provide an index command for initial repository indexing and SHALL prompt users to enable neural navigation when neural configuration has not yet been initialized.

#### Scenario: Run index command
- **WHEN** the user runs `flashgrep index`
- **THEN** it SHALL scan and index the current directory

#### Scenario: Index specific directory
- **WHEN** the user runs `flashgrep index /path/to/repo`
- **THEN** it SHALL index the specified directory

#### Scenario: Show indexing progress
- **WHEN** the index command runs
- **THEN** it SHALL display real-time progress to stdout

#### Scenario: Prompt to enable neural navigation on first index
- **WHEN** index starts and neural navigation configuration is unset in interactive mode
- **THEN** the CLI SHALL prompt whether to enable neural navigation
- **AND** if the user declines, indexing SHALL continue with lexical mode only

#### Scenario: Success exit code
- **WHEN** indexing completes successfully
- **THEN** it SHALL exit with code 0

#### Scenario: Error exit code
- **WHEN** indexing encounters a fatal error
- **THEN** it SHALL exit with a non-zero code and print the error
