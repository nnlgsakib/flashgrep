## ADDED Requirements

### Requirement: Chunk metadata storage
The metadata store SHALL persist chunk information for each indexed file.

#### Scenario: Store chunk record
- **WHEN** a file is chunked
- **THEN** it SHALL store: file_path, start_line, end_line, content_hash, last_modified

#### Scenario: Update chunk on re-index
- **WHEN** a file is re-indexed
- **THEN** it SHALL update existing chunk records or create new ones

#### Scenario: Delete chunk on file removal
- **WHEN** a file is deleted
- **THEN** it SHALL remove all chunks for that file path

### Requirement: Symbol metadata storage
The metadata store SHALL persist detected symbols for structural search.

#### Scenario: Store symbol record
- **WHEN** a symbol is detected in a chunk
- **THEN** it SHALL store: symbol_name, file_path, line_number, symbol_type

#### Scenario: Query symbols by name
- **WHEN** searching for a symbol
- **THEN** it SHALL return all records matching the symbol name

#### Scenario: Delete symbols on file removal
- **WHEN** a file is deleted
- **THEN** it SHALL remove all symbols associated with that file

### Requirement: File metadata storage
The metadata store SHALL persist file-level information.

#### Scenario: Store file record
- **WHEN** a file is indexed
- **THEN** it SHALL store: file_path, file_size, last_modified, language (if detectable)

#### Scenario: Query file by path
- **WHEN** looking up a specific file
- **THEN** it SHALL return the file metadata record

### Requirement: Index statistics
The metadata store SHALL support aggregate queries for index statistics.

#### Scenario: Count total files
- **WHEN** stats are requested
- **THEN** it SHALL return the total count of indexed files

#### Scenario: Count total chunks
- **WHEN** stats are requested
- **THEN** it SHALL return the total count of chunks

#### Scenario: Get last update time
- **WHEN** stats are requested
- **THEN** it SHALL return the timestamp of the most recent index update
