## MODIFIED Requirements

### Requirement: Initial indexing progress
The indexing engine SHALL report progress during the indexing operation and SHALL check model availability at startup to support neural workflows without blocking lexical indexing.

#### Scenario: Display progress
- **WHEN** the index command runs
- **THEN** it SHALL print progress showing: files scanned, files indexed, current file, estimated completion

#### Scenario: Prompt decision when model is missing
- **WHEN** indexing starts and model cache is missing
- **THEN** it SHALL request a yes/no decision for model download before indexing work begins
- **AND** declining the prompt SHALL continue indexing without model download

#### Scenario: Create flashgrep directory
- **WHEN** indexing completes successfully
- **THEN** it SHALL create the `.flashgrep/` directory with: text_index/, metadata.db, config.json
