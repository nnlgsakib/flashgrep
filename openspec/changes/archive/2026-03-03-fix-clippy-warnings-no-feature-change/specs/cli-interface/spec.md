## ADDED Requirements

### Requirement: CLI behavior parity during lint-compliance refactors
Changes made to satisfy strict clippy linting SHALL NOT alter existing CLI command names, argument handling, output contracts, or exit-code behavior.

#### Scenario: Existing command contracts remain stable
- **WHEN** a user runs previously supported CLI commands with valid or invalid arguments after lint-compliance updates
- **THEN** command parsing, output structure, and exit-code semantics SHALL match pre-change behavior
