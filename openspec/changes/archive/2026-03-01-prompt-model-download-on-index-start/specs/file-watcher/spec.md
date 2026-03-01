## MODIFIED Requirements

### Requirement: Initial indexing on watcher start
The file watcher SHALL perform an initial scan and index of all files when it starts and SHALL honor model prompt decisions when model assets are missing.

#### Scenario: Scan repository on startup
- **WHEN** the file watcher is started for a repository
- **THEN** it SHALL begin scanning all files in the repository
- **AND** it SHALL build an index of current file state

#### Scenario: Prompt before initial indexing when model is missing
- **WHEN** watcher startup enters initial indexing and model cache is missing
- **THEN** it SHALL prompt user for model download confirmation
- **AND** if declined, initial indexing SHALL continue without model download

#### Scenario: Progress logging for large repositories
- **WHEN** scanning a repository with more than 1000 files
- **THEN** it SHALL log progress periodically (e.g., every 1000 files)
- **AND** it SHALL indicate that initial indexing is in progress
