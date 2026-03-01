## MODIFIED Requirements

### Requirement: Initial indexing progress
The indexing engine SHALL report progress during the indexing operation and SHALL initialize artifacts required for both lexical and neural retrieval.

#### Scenario: Display progress
- **WHEN** the index command runs
- **THEN** it SHALL print progress showing: files scanned, files indexed, current file, estimated completion

#### Scenario: Create flashgrep directory
- **WHEN** indexing completes successfully
- **THEN** it SHALL create the `.flashgrep/` directory with: text_index/, metadata.db, config.json
- **AND** it SHALL initialize vector and model cache paths used for neural retrieval under `.flashgrep/`

## ADDED Requirements

### Requirement: Embedding generation and storage
The indexing engine SHALL generate embeddings from indexed chunks and SHALL persist vector metadata required for semantic retrieval.

#### Scenario: Generate vectors for indexed chunks
- **WHEN** a file is indexed or re-indexed
- **THEN** the engine SHALL generate embeddings for resulting chunks
- **AND** it SHALL store vectors keyed to file path, line range, and content hash

#### Scenario: Incremental updates replace stale vectors
- **WHEN** a file changes, is removed, or becomes ignored
- **THEN** the engine SHALL remove stale vectors for affected chunks
- **AND** it SHALL persist replacement vectors only for current indexable content
