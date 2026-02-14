## Context

Flashgrep is a greenfield Rust project building a high-performance local code indexing engine. The system must handle 1M+ lines of code, provide sub-50ms query responses, and operate entirely locally without cloud dependencies. This is a new architectural component, not a modification to existing code.

**Current State**: No existing codebase - building from scratch
**Constraints**: 
- Must be a single static binary
- Memory usage under 500MB
- Cross-platform (Linux, Mac, Windows)
- No runtime dependencies
- MCP-compatible server interface

**Stakeholders**: 
- End users (developers running the CLI)
- Coding agents (consuming MCP server API)
- LLM systems (receiving token-efficient search results)

## Goals / Non-Goals

**Goals:**
- Index any repository regardless of language
- Full-text search via Tantivy with custom ranking
- Structural search using regex-based symbol detection
- File watching with incremental re-indexing
- MCP server exposing JSON-RPC methods
- Token-efficient responses (exact slices, not full files)
- CLI with `index` and `start` commands
- Portable `.flashgrep/` directory layout
- Graceful shutdown and error handling
- Support `.flashgrepignore` for custom ignore patterns

**Non-Goals:**
- Language-specific AST parsing (using heuristics only)
- Vector/semantic search (future extensibility only)
- Multi-user or team collaboration features
- Web UI or visual components
- Git integration beyond .gitignore support
- Code execution or modification capabilities
- Nested `.flashgrepignore` files (root only for v1)

## Decisions

### 1. Use Tantivy for Full-Text Search
**Decision**: Use tantivy crate as the search backend
**Rationale**: 
- Purpose-built for full-text search with Rust bindings
- Supports custom scoring and tokenization
- Proven performance for large document sets
- Single-process design fits our constraints

**Alternatives considered**:
- Bleve (Go-based, cross-language complexity)
- Custom inverted index (reinventing the wheel)
- Elasticsearch (requires external service)

### 2. SQLite for Metadata Storage (Optimized)
**Decision**: Use rusqlite with connection pooling and batch operations
**Rationale**:
- Embedded, zero-config database
- ACID compliance for index consistency
- Cross-platform portable files
- Can handle 50k+ inserts/second with proper tuning
- Optimizations applied:
  - **Connection pooling** (r2d2): Reuse connections instead of opening/closing
  - **Batch inserts**: Transactions with multiple INSERTs (50-100x faster)
  - **WAL mode**: Write-Ahead Logging for concurrent reads/writes
  - **100MB cache**: Larger page cache reduces disk I/O
  - **Memory-mapped I/O**: 256MB mmap for faster access
  - **Synchronous=NORMAL**: Balanced safety/performance with WAL

**Performance**: ~2.6 seconds to index 1,576 files (60k+ rows)

**Alternatives considered**:
- Flat JSON files (no query capabilities)
- Sled (pure Rust, but less mature)
- PostgreSQL (external dependency)
- RocksDB (fast but adds 5MB+ to binary)

### 3. Regex-Based Symbol Detection
**Decision**: Use regex patterns instead of language parsers
**Rationale**:
- True language agnosticism
- No parser dependencies or compilation overhead
- Fast enough for indexing throughput
- Heuristic patterns cover 80% of common cases

**Alternatives considered**:
- Tree-sitter (requires language grammars)
- Custom parsers per language (maintenance burden)
- ctags integration (external dependency)

### 4. Notify Crate for File Watching
**Decision**: Use notify crate for cross-platform file system events
**Rationale**:
- Unified API across Linux (inotify), Mac (FSEvents), Windows (ReadDirectoryChanges)
- Handles debouncing and event coalescing
- Well-maintained with active community

### 5. Tokio for Async Runtime
**Decision**: Use tokio as the async runtime
**Rationale**:
- Industry standard for Rust async
- Required by MCP server JSON-RPC handling
- Enables concurrent indexing and serving
- Mature ecosystem with good tooling

### 6. MCP over JSON-RPC
**Decision**: Implement MCP protocol over JSON-RPC
**Rationale**:
- Standard protocol for AI tool integration
- Well-documented schema
- Easy for coding agents to consume
- Can use Unix sockets (fast) or TCP (portable)

### 7. Gitignore-Style Pattern Matching for .flashgrepignore
**Decision**: Use gitignore-style glob patterns for `.flashgrepignore` files
**Rationale**:
- Familiar format to developers (same as .gitignore)
- Supports negation patterns (`!`), directory-only patterns (`/`), and wildcards
- Well-documented and understood semantics
- Can reuse or adapt existing glob matching libraries

**Alternatives considered**:
- Regular expressions (too complex for users)
- Custom pattern syntax (learning curve, inconsistent with ecosystem)
- Full .gitignore support including nested files (overkill for v1)

**Scope**: Support patterns in root `.flashgrepignore` only (not nested directories)

## Risks / Trade-offs

**[Risk] Regex patterns may miss complex language constructs**
→ Mitigation: Document limitations; design for pluggable parser extension in future

**[Risk] Tantivy index size for very large repos**
→ Mitigation: Implement index pruning and chunk size limits; monitor disk usage

**[Risk] File watcher may miss rapid changes during heavy I/O**
→ Mitigation: Periodic full re-index option; expose manual re-index command

**[Risk] Cross-platform socket path handling**
→ Mitigation: Abstract transport layer; prefer TCP on Windows, Unix sockets on Unix

**[Risk] SQLite concurrency during indexing and querying**
→ Mitigation: Use WAL mode; separate read/write connection pools; batch updates

**[Trade-off] Memory vs Speed**
→ Accept 500MB memory limit to achieve 50ms query times; use streaming for large results

## Migration Plan

**Deployment Steps**:
1. Build release binary with `cargo build --release`
2. Strip and optimize binary size
3. Distribute via GitHub releases
4. Users install binary to PATH

**Rollback Strategy**:
- Delete `.flashgrep/` directory to clear index
- Binary is self-contained - no system changes to revert

**First-Time Setup**:
```bash
flashgrep index  # Creates .flashgrep/ in current directory
flashgrep start  # Starts daemon with MCP server
```

## Open Questions

1. **Chunk size optimization**: Is 300 lines the right max, or should it be byte-based?
2. **Index versioning**: How to handle index format changes between versions?
3. **Symbol ranking weights**: What scoring factors matter most for relevance?
4. **MCP auth**: Should the server require authentication tokens?
