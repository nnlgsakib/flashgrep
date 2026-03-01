## MODIFIED Requirements

### Requirement: Initial indexing progress
The indexing engine SHALL report progress during the indexing operation and SHALL initialize artifacts required for both lexical and neural retrieval.

#### Scenario: Display progress
- **WHEN** the index command runs
- **THEN** it SHALL print progress showing: files scanned, files indexed, current file, estimated completion

#### Scenario: Create flashgrep directory
- **WHEN** indexing completes successfully
- **THEN** it SHALL create the `.flashgrep/` directory with: text_index/, metadata.db, config.json
- **AND** it SHALL initialize neural artifacts for local model and vectors under `.flashgrep/` using deterministic paths
