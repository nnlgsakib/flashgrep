## ADDED Requirements

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
