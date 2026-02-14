## ADDED Requirements

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
