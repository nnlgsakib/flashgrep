## ADDED Requirements

### Requirement: Ignore file change reconciliation
The watcher SHALL react to `.flashgrepignore` updates by reconciling indexed content.

#### Scenario: Ignore file modified
- **WHEN** `.flashgrepignore` is created or modified
- **THEN** watcher SHALL reload ignore patterns
- **AND** it SHALL trigger reconciliation for already indexed files

#### Scenario: Newly ignored paths are pruned
- **WHEN** reconciliation finds indexed files now matching ignore rules
- **THEN** watcher/indexer SHALL remove those files from index and metadata
- **AND** subsequent searches SHALL NOT return those files

### Requirement: Ignore reconciliation observability
The watcher SHALL provide user-visible diagnostics for ignore reconciliation.

#### Scenario: Reconciliation summary
- **WHEN** ignore reconciliation completes
- **THEN** it SHALL log counts for files removed and files kept
