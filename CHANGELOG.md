# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-14

### Added
- Initial release of Flashgrep
- **CLI Commands**:
  - `flashgrep index` - Index a repository with incremental updates
  - `flashgrep index --force` - Force full re-index
  - `flashgrep start` - Start daemon with file watcher and MCP server
  - `flashgrep stats` - Show index statistics
  - `flashgrep clear` - Clear the index
- **Core Features**:
  - Language-agnostic code indexing using regex-based heuristics
  - Full-text search via Tantivy with custom ranking
  - Symbol detection (functions, classes, imports, exports, routes, SQL)
  - Intelligent file chunking (blank lines, bracket balance, 300 line max)
  - Incremental indexing - only re-indexes changed files
  - File watching with auto-reindex on changes
  - SQLite metadata store with connection pooling
  - Batch inserts for 50-100x faster indexing
  - `.flashgrepignore` support for custom ignore patterns
- **MCP Server**:
  - JSON-RPC API on TCP port 7777
  - Methods: `query`, `get_slice`, `get_symbol`, `list_files`, `stats`
  - Integration with coding agents
- **Performance**:
  - Index 1,800+ files in ~2 seconds
  - Incremental updates in ~0.3 seconds
  - Sub-50ms query responses
  - Memory usage under 200MB
- **Documentation**:
  - Comprehensive README with installation and usage
  - BUILD.md with platform-specific instructions
  - API documentation for MCP server
  - 38 unit and integration tests

### Technical Details
- Rust 2021 edition
- Async runtime with Tokio
- Tantivy 0.21 for full-text search
- SQLite with WAL mode and connection pooling (r2d2)
- Cross-platform support (Linux, macOS, Windows)
- Single static binary distribution

[0.1.0]: https://github.com/nnlgsakib/flashgrep/releases/tag/v0.1.0
