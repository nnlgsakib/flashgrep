## ADDED Requirements

### Requirement: Bulk prune for ignored paths
The metadata store SHALL support bulk removal of indexed records for paths that become ignored.

#### Scenario: Remove records for newly ignored files
- **WHEN** reconciliation provides a set of file paths now ignored
- **THEN** the metadata store SHALL delete associated file, chunk, and symbol records for those paths

#### Scenario: Idempotent prune execution
- **WHEN** prune is executed multiple times for the same ignored path set
- **THEN** it SHALL complete successfully without inconsistent state
- **AND** resulting metadata SHALL remain correct
