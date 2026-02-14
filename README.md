# Flashgrep

A high-performance, local code indexing engine designed for LLM coding agents. Flashgrep provides blazing fast text and structural search across any codebase with minimal memory footprint.

## Features

- **Language Agnostic**: Works with any programming language using regex-based heuristics
- **Blazing Fast**: Sub-50ms query responses, incremental indexing in <1 second
- **Minimal Memory**: Under 500MB memory usage for large repositories
- **Fully Local**: No cloud dependencies, all data stays on your machine
- **Token Efficient**: Returns exact code slices, not full files
- **Single Binary**: Distributed as one static binary, no runtime dependencies
- **MCP Compatible**: JSON-RPC server for integration with coding agents

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/flashgrep.git
cd flashgrep

# Build release binary
cargo build --release

# Install to PATH
cp target/release/flashgrep /usr/local/bin/
```

### Pre-built Binaries

Download pre-built binaries from the [releases page](https://github.com/yourusername/flashgrep/releases).

## Quick Start

```bash
# Navigate to your project
cd /path/to/your/project

# Create initial index
flashgrep index

# Start the daemon (file watcher + MCP server)
flashgrep start
```

## Usage

### CLI Commands

#### `flashgrep index [PATH]`

Index a repository for searching. If PATH is not provided, indexes the current directory.

```bash
# Index current directory
flashgrep index

# Index specific directory
flashgrep index /path/to/project
```

Features:
- **Incremental indexing**: Only re-indexes changed files
- **Fast**: Indexes 1,500+ files in under 3 seconds
- **Smart filtering**: Ignores `target/`, `node_modules/`, `.git/`, etc.

#### `flashgrep start [PATH]`

Start the daemon with file watcher and MCP server.

```bash
# Start daemon for current directory
flashgrep start

# Start with specific directory
flashgrep start /path/to/project
```

The daemon:
- Watches files for changes and auto-updates index
- Runs MCP server on `localhost:7777`
- Supports graceful shutdown (Ctrl+C)

### MCP Server API

The MCP server exposes JSON-RPC methods for coding agents. For comprehensive documentation including best practices, workflows, and examples, see the [AI Agent Skill Guide](.opencode/skills/flashgrep-mcp/SKILL.md).

**Available Methods:**

#### `query(text, limit)`

Search for text in the indexed codebase.

```json
{
  "jsonrpc": "2.0",
  "method": "query",
  "params": {
    "text": "function main",
    "limit": 10
  },
  "id": 1
}
```

#### `get_slice(file_path, start_line, end_line)`

Retrieve specific lines from a file.

```json
{
  "jsonrpc": "2.0",
  "method": "get_slice",
  "params": {
    "file_path": "src/main.rs",
    "start_line": 1,
    "end_line": 50
  },
  "id": 2
}
```

#### `get_symbol(symbol_name)`

Find all occurrences of a symbol.

```json
{
  "jsonrpc": "2.0",
  "method": "get_symbol",
  "params": {
    "symbol_name": "main"
  },
  "id": 3
}
```

#### `list_files()`

List all indexed files.

```json
{
  "jsonrpc": "2.0",
  "method": "list_files",
  "params": {},
  "id": 4
}
```

#### `stats()`

Get index statistics.

```json
{
  "jsonrpc": "2.0",
  "method": "stats",
  "params": {},
  "id": 5
}
```

## Configuration

### `.flashgrepignore`

Create a `.flashgrepignore` file in your project root to exclude files/directories from indexing. Uses gitignore-style patterns:

```
# Ignore all log files
*.log

# Ignore build directories
build/
dist/

# Ignore specific files
config.local.json

# Re-include specific files
!important.log
```

### Config File

The config is stored in `.flashgrep/config.json`:

```json
{
  "version": "0.1.0",
  "mcp_port": 7777,
  "use_unix_socket": false,
  "max_file_size": 2097152,
  "max_chunk_lines": 300,
  "extensions": ["rs", "js", "ts", "py", "go", "json", "md", "yaml", "toml"],
  "ignored_dirs": [".git", "node_modules", "target", "dist", "build", "vendor"],
  "debounce_ms": 500
}
```

## Architecture

### Components

- **File Scanner**: Recursively finds indexable files, respects `.flashgrepignore`
- **Chunker**: Splits files into logical chunks (max 300 lines, preserves bracket balance)
- **Symbol Detector**: Regex-based detection of functions, classes, imports, etc.
- **Tantivy Index**: Full-text search engine with custom ranking
- **SQLite Store**: Metadata storage with connection pooling and batch inserts
- **File Watcher**: Incremental re-indexing with debouncing
- **MCP Server**: JSON-RPC over TCP for agent integration

### Index Structure

```
.flashgrep/
├── text_index/        # Tantivy full-text index
├── metadata.db        # SQLite database (chunks, symbols, file metadata)
├── config.json        # Configuration
└── vectors/           # Reserved for future use
```

## Performance

Benchmarks on a typical codebase (1,576 files, ~50k lines):

- **Initial indexing**: ~2.6 seconds
- **Incremental indexing**: ~0.35 seconds (only changed files)
- **Query response**: <50ms
- **Memory usage**: ~150MB
- **Index size**: ~50MB

## File Support

### Indexed Extensions

- Rust (`.rs`)
- JavaScript/TypeScript (`.js`, `.ts`)
- Python (`.py`)
- Go (`.go`)
- Solidity (`.sol`)
- JSON (`.json`)
- Markdown (`.md`)
- YAML (`.yaml`, `.yml`)
- TOML (`.toml`)

### Ignored Directories

- `.git`
- `node_modules`
- `target`
- `dist`
- `build`
- `vendor`
- `.flashgrep`

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run -- index
```

### Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── cli/              # Command-line interface
├── config/           # Configuration management
├── db/               # SQLite database layer
├── index/            # File scanning and indexing
├── chunking/         # File chunking logic
├── symbols/          # Symbol detection
├── search/           # Search engine
├── watcher/          # File system watcher
└── mcp/              # MCP server
```

## Troubleshooting

### Index is slow

Use the release build:
```bash
cargo build --release
./target/release/flashgrep index
```

### Out of memory

Reduce cache size in `config.json`:
```json
{
  "max_file_size": 1048576
}
```

### Lock errors

Delete the index and re-index:
```bash
rm -rf .flashgrep
flashgrep index
```

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please read CONTRIBUTING.md for guidelines.

## Roadmap

- [ ] Plugin-based language parsers
- [ ] Team shared index
- [ ] Visual graph UI
- [ ] Call graph engine
- [ ] Semantic embeddings
- [ ] Refactor impact analysis
