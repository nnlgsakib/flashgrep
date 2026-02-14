## 1. Project Setup and Dependencies

- [x] 1.1 Initialize Rust project with `cargo new flashgrep --bin`
- [x] 1.2 Add core dependencies to Cargo.toml: tantivy, rusqlite, notify, clap, tokio, serde, serde_json, regex
- [x] 1.3 Set up project structure: src/{cli,index,search,watcher,mcp,db,symbols,chunking,config}
- [x] 1.4 Create lib.rs with module declarations
- [x] 1.5 Configure release profile for static binary and size optimization

## 2. Configuration and Data Models

- [x] 2.1 Define Config struct with serde serialization for config.json
- [x] 2.2 Define Chunk struct: file_path, start_line, end_line, content_hash, content
- [x] 2.3 Define Symbol struct: symbol_name, file_path, line_number, symbol_type
- [x] 2.4 Define FileMetadata struct: file_path, file_size, last_modified, language
- [x] 2.5 Define SearchResult struct with all required fields
- [x] 2.6 Implement .flashgrep/ directory layout and path utilities

## 3. Metadata Store (SQLite)

- [x] 3.1 Create Database struct wrapping rusqlite connection
- [x] 3.2 Implement schema initialization for chunks table
- [x] 3.3 Implement schema initialization for symbols table
- [x] 3.4 Implement schema initialization for files table
- [x] 3.5 Implement insert_chunk() method with conflict handling
- [x] 3.6 Implement insert_symbol() method
- [x] 3.7 Implement insert_file() method
- [x] 3.8 Implement delete_file_chunks() method
- [x] 3.9 Implement delete_file_symbols() method
- [x] 3.10 Implement get_stats() for file count, chunk count, last update
- [x] 3.11 Enable WAL mode for concurrent reads/writes
- [x] 3.12 Add error handling for database operations

## 4. File Filtering and Chunking

- [x] 4.1 Implement should_ignore_directory() for ignored dirs
- [x] 4.2 Implement should_index_file() for file extension filtering
- [x] 4.3 Implement is_binary_file() check
- [x] 4.4 Implement is_oversized_file() check for 2MB limit
- [x] 4.5 Create FileScanner to recursively find indexable files
- [x] 4.6 Implement FlashgrepIgnore struct to parse .flashgrepignore files
- [x] 4.7 Implement pattern matching for gitignore-style globs (wildcards, negation)
- [x] 4.8 Handle directory-only patterns (trailing `/`)
- [x] 4.9 Handle comment lines (starting with `#`)
- [x] 4.10 Handle blank lines in ignore file
- [x] 4.11 Integrate ignore patterns into FileScanner
- [x] 4.12 Implement chunk_by_blank_lines() splitting logic
- [x] 4.13 Implement bracket_balance_check() for keeping blocks together
- [x] 4.14 Implement enforce_max_chunk_size() with 300 line limit
- [x] 4.15 Create Chunker that combines all chunking strategies
- [x] 4.16 Implement content_hash calculation for deduplication

## 5. Symbol Detection

- [x] 5.1 Define regex patterns for function definitions
- [x] 5.2 Define regex patterns for class/struct definitions
- [x] 5.3 Define regex patterns for import statements
- [x] 5.4 Define regex patterns for export statements
- [x] 5.5 Define regex patterns for route definitions
- [x] 5.6 Define regex patterns for SQL queries
- [x] 5.7 Define regex patterns for visibility markers
- [x] 5.8 Create SymbolDetector with all regex patterns
- [x] 5.9 Implement detect_in_chunk() method
- [x] 5.10 Implement extract_symbol_name() helper

## 6. Indexing Engine

- [x] 6.1 Create Indexer struct with Database and Tantivy dependencies
- [x] 6.2 Initialize Tantivy index schema with required fields
- [x] 6.3 Implement index_file() method: scan, chunk, detect symbols, store
- [x] 6.4 Implement add_chunk_to_tantivy() for text indexing
- [x] 6.5 Implement index_repository() for full directory scanning
- [x] 6.6 Add progress reporting with file count and ETA
- [x] 6.7 Implement index existence check
- [x] 6.8 Implement clear_index() for full re-index option

## 7. Search Engine

- [x] 7.1 Create Searcher struct wrapping Tantivy searcher
- [x] 7.2 Implement query() method with text search
- [x] 7.3 Implement custom scorer for symbol match boost
- [x] 7.4 Add proximity ranking to query parser
- [x] 7.5 Add recency boost based on file modification time
- [x] 7.6 Add file depth penalty for ranking
- [x] 7.7 Implement limit parameter for result count
- [x] 7.8 Implement get_slice() for exact line range retrieval
- [x] 7.9 Implement get_symbol() for symbol lookup from SQLite
- [x] 7.10 Optimize query response time under 50ms

## 8. File Watcher

- [x] 8.1 Create FileWatcher struct using notify crate
- [x] 8.2 Implement watch_repository() to start file system monitoring
- [x] 8.3 Handle Create events with debouncing
- [x] 8.4 Handle Modify events with debouncing
- [x] 8.5 Handle Remove events to delete from index
- [x] 8.6 Implement debounce logic (500ms window)
- [x] 8.7 Handle permission errors gracefully
- [x] 8.8 Handle file move during indexing edge cases

## 9. MCP Server

- [x] 9.1 Create McpServer struct with JSON-RPC handler
- [x] 9.2 Implement transport abstraction (TCP and Unix socket)
- [x] 9.3 Implement JSON-RPC request parsing
- [x] 9.4 Implement JSON-RPC response formatting
- [x] 9.5 Implement query() method handler
- [x] 9.6 Implement get_slice() method handler
- [x] 9.7 Implement get_symbol() method handler
- [x] 9.8 Implement list_files() method handler
- [x] 9.9 Implement stats() method handler
- [x] 9.10 Ensure minimal response format (no full content in query results)
- [x] 9.11 Add error response handling for missing files

## 10. CLI Interface

- [x] 10.1 Set up clap with derive macros for CLI structure
- [x] 10.2 Implement `flashgrep index` command
- [x] 10.3 Implement `flashgrep index <path>` for specific directory
- [x] 10.4 Implement `flashgrep start` command
- [x] 10.5 Implement progress display for index command
- [x] 10.6 Print server address on daemon start
- [x] 10.7 Implement graceful shutdown on SIGINT/SIGTERM
- [x] 10.8 Add --help and --version flags
- [x] 10.9 Set appropriate exit codes (0 for success, non-zero for errors)
- [x] 10.10 Add .flashgrepignore documentation to help output

## 11. Integration and Main

- [x] 11.1 Wire up CLI commands to actual implementations
- [x] 11.2 Implement main() with async runtime setup
- [x] 11.3 Add logging initialization with tracing
- [x] 11.4 Set up error handling and panic hooks
- [x] 11.5 Implement graceful shutdown coordination
- [x] 11.6 Ensure single binary compilation

## 12. Testing and Quality

- [x] 12.1 Create test repository with multiple languages
- [x] 12.2 Create test repository with 10k+ lines
- [x] 12.3 Create test repository with nested directories
- [x] 12.4 Write unit tests for file filtering logic
- [x] 12.5 Write unit tests for chunking algorithm
- [x] 12.6 Write unit tests for symbol detection regexes
- [x] 12.7 Write integration test for full indexing workflow
- [x] 12.8 Write integration test for search functionality
- [x] 12.9 Write integration test for file watcher
- [x] 12.10 Verify fast indexing performance (1M+ lines)
- [x] 12.11 Verify query response under 50ms
- [x] 12.12 Add CI workflow for automated testing

## 13. Documentation and Build

- [x] 13.1 Write comprehensive README.md with installation
- [x] 13.2 Document CLI usage with examples
- [x] 13.3 Document MCP server protocol and methods
- [x] 13.4 Document .flashgrepignore file format and examples
- [x] 13.5 Add inline code documentation (rustdoc) - Fixed all compiler warnings
- [x] 13.6 Create BUILD.md with compilation instructions
- [x] 13.7 Test cross-platform builds (Linux, Mac, Windows)
- [x] 13.8 Optimize binary size with strip and LTO
- [x] 13.9 Create release script for GitHub releases
