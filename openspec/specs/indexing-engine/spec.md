## ADDED Requirements

### Requirement: File scanning and filtering
The indexing engine SHALL recursively scan the current directory for indexable files.

#### Scenario: Skip ignored directories
- **WHEN** the indexer scans a repository
- **THEN** it SHALL skip directories: .git, node_modules, target, dist, build, vendor

#### Scenario: Filter by file extension
- **WHEN** the indexer encounters a file
- **THEN** it SHALL index files with extensions: .go, .rs, .js, .ts, .py, .sol, .json, .md, .yaml, .toml

#### Scenario: Skip binary and oversized files
- **WHEN** the indexer encounters a file over 2MB or binary content
- **THEN** it SHALL skip the file and log the skip reason

### Requirement: File chunking
The indexing engine SHALL split files into logical chunks for storage and search.

#### Scenario: Chunk by blank lines
- **WHEN** chunking a source file
- **THEN** it SHALL split chunks at blank lines

#### Scenario: Preserve bracket-balanced blocks
- **WHEN** a code block has balanced brackets
- **THEN** it SHALL keep the entire block in a single chunk

#### Scenario: Enforce maximum chunk size
- **WHEN** a chunk exceeds 300 lines
- **THEN** it SHALL force split at the nearest logical boundary

### Requirement: Initial indexing progress
The indexing engine SHALL report progress during the indexing operation.

#### Scenario: Display progress
- **WHEN** the index command runs
- **THEN** it SHALL print progress showing: files scanned, files indexed, current file, estimated completion

#### Scenario: Create flashgrep directory
- **WHEN** indexing completes successfully
- **THEN** it SHALL create the `.flashgrep/` directory with: text_index/, metadata.db, config.json

### Requirement: Flashgrepignore support
The indexing engine SHALL respect `.flashgrepignore` files for custom ignore patterns.

#### Scenario: Parse flashgrepignore file
- **WHEN** a `.flashgrepignore` file exists in the repository root
- **THEN** it SHALL parse the file and apply gitignore-style patterns

#### Scenario: Ignore patterns matching
- **WHEN** a file or directory matches a pattern in `.flashgrepignore`
- **THEN** it SHALL skip that file or directory during indexing

#### Scenario: Pattern negation
- **WHEN** a pattern starts with `!`
- **THEN** it SHALL un-ignore previously ignored files matching the negation pattern

#### Scenario: Directory patterns
- **WHEN** a pattern ends with `/`
- **THEN** it SHALL only match directories, not files

#### Scenario: Comment lines
- **WHEN** a line starts with `#`
- **THEN** it SHALL treat that line as a comment and ignore it

#### Scenario: Blank lines
- **WHEN** a line is empty or contains only whitespace
- **THEN** it SHALL ignore that line

### Requirement: Database clear_all method
The system SHALL provide a method to clear all data from the metadata database.

#### Scenario: Clear all tables
- **WHEN** the clear_all() method is called
- **THEN** all records in the files table SHALL be deleted
- **AND** all records in the chunks table SHALL be deleted
- **AND** all records in the symbols table SHALL be deleted
- **AND** foreign key constraints SHALL be handled via CASCADE

### Requirement: Index clearing synchronization
The system SHALL clear both text index and metadata database when clear_index() is called.

#### Scenario: Clear command execution
- **WHEN** flashgrep clear is executed
- **THEN** the Tantivy text index SHALL be cleared
- **AND** the SQLite metadata database SHALL be cleared
- **AND** subsequent indexing SHALL treat all files as new

### Requirement: Consistent search results
The system SHALL provide consistent results between text search and symbol search.

#### Scenario: After index rebuild
- **WHEN** a repository is indexed after clearing
- **THEN** text search SHALL return results for indexed content
- **AND** symbol search SHALL return results for indexed content
- **AND** both searches SHALL work on the same set of files
