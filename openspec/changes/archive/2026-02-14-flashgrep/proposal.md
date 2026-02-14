## Why

Build a high-performance local code indexing engine named flashgrep that provides blazing fast text and structural search across any codebase. Current code search tools are either slow, memory-intensive, cloud-dependent, or return too much irrelevant context for LLM coding agents. We need a deterministic, token-efficient, fully local solution that can index millions of lines and return precise results in under 50ms.

## What Changes

- Create a Rust-based CLI tool distributed as a single static binary
- Implement `flashgrep index` command for initial repository indexing
- Implement `flashgrep start` command for daemon mode with file watching
- Build tantivy-based text search engine with custom ranking
- Create SQLite metadata store for chunks and symbols
- Implement language-agnostic structural heuristics using regex patterns
- Build MCP-compatible JSON-RPC server (Unix socket / TCP)
- Add intelligent file chunking by logical blocks
- Create incremental indexing on file changes via notify crate
- Store index in `.flashgrep/` directory with portable layout
- Support `.flashgrepignore` file for custom ignore patterns (like `.gitignore`)
- Support cross-platform deployment (Linux, Mac, Windows)

## Capabilities

### New Capabilities
- `indexing-engine`: Core indexing system with file scanning, filtering, and chunking
- `search-engine`: Text and structural search with relevance ranking
- `file-watcher`: Incremental re-indexing on file system changes
- `mcp-server`: JSON-RPC server exposing query methods for coding agents
- `cli-interface`: Command-line interface with index and start commands
- `metadata-store`: SQLite-based storage for chunks, symbols, and file metadata
- `symbol-detection`: Language-agnostic pattern matching for code structures
- `ignore-patterns`: Support for `.flashgrepignore` files with gitignore-style patterns

### Modified Capabilities
- None (new project)

## Impact

- New Rust project with modular architecture
- Dependencies: tantivy, sqlite, notify, clap, tokio, serde
- Creates `.flashgrep/` directory in project roots (gitignore recommended)
- MCP server runs on localhost:7777 or Unix socket
- Single static binary with no runtime dependencies
- Memory usage capped at 500MB for large repositories
- No cloud dependencies - fully local operation
