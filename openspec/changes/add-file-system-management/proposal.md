## Why

Currently, working with the file system requires relying on coding agents' naive read/write operations, which are inefficient and lack proper batching, caching, and atomic operations. Users need a dedicated file system management layer that provides efficient, batched, and intelligent file operations without requiring manual coding agent interactions.

## What Changes

- **New file system management commands**: Add native commands for common file operations (read, write, copy, move, delete, batch operations)
- **Batch operation support**: Enable efficient multi-file operations with single commands instead of sequential individual operations
- **Intelligent caching layer**: Cache file reads to avoid redundant disk access during the same session
- **Atomic transaction support**: Group multiple file operations into atomic units that either all succeed or all fail
- **File watching capabilities**: Monitor file changes without polling
- **Efficient search and globbing**: Built-in support for finding files by patterns

## Capabilities

### New Capabilities
- `fs-read`: Efficient file reading with caching and multiple file support
- `fs-write`: Batch write operations with atomic transaction support
- `fs-batch`: Multi-file operations (copy, move, delete) in single commands
- `fs-search`: Pattern-based file searching and globbing
- `fs-watch`: File system watching for change notifications

### Modified Capabilities
- None

## Impact

- New command interface for file system operations
- Potential dependency on file system watching libraries
- Changes to how agents interact with file system (should use new commands)
- Improved performance for file-heavy workflows
