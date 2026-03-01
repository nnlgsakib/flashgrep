## MODIFIED Requirements

### Requirement: Help and version
The CLI SHALL provide standard help and version commands, including a `version` subcommand and enriched platform details.

#### Scenario: Show help
- **WHEN** the user runs `flashgrep --help` or `flashgrep -h`
- **THEN** it SHALL display usage information for all commands

#### Scenario: Show version via flags
- **WHEN** the user runs `flashgrep --version` or `flashgrep -V`
- **THEN** it SHALL display the Flashgrep version, operating system, and CPU architecture

#### Scenario: Show version via command
- **WHEN** the user runs `flashgrep version`
- **THEN** it SHALL display the same version information as `--version` and `-V`
