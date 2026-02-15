# Initial Indexing on Watcher Start

## Overview

Flashgrep now performs an **initial index scan** when the file watcher starts. This feature ensures that any changes made while the watcher was offline are detected and properly indexed, providing a complete and accurate view of your repository state.

## How It Works

When you start the file watcher with `flashgrep start`, the following happens:

1. **File system watcher starts immediately** - The watcher begins monitoring for real-time changes right away (non-blocking)
2. **Initial scan runs in the background** - All files in the repository are scanned asynchronously
3. **Change detection** - The scanner compares the current state with the previous index to detect:
   - **Added files** - New files created while the watcher was offline
   - **Modified files** - Files changed since the last scan
   - **Deleted files** - Files removed while the watcher was offline
4. **Synthetic events** - Detected changes trigger the same indexing pipeline as real-time events
5. **Index state persisted** - The updated index is saved to `.flashgrep/index-state.json`

## Benefits

- **No missed changes** - Changes made while the watcher was stopped are automatically detected
- **Non-blocking** - File watching starts immediately; scanning happens in the background
- **Efficient** - Only changed files are re-indexed; unchanged files are skipped
- **Resumable** - If the watcher crashes, the partial state is preserved for the next start

## Configuration

You can customize the initial indexing behavior in your `.flashgrep/config.json`:

```json
{
  "enable_initial_index": true,
  "progress_interval": 1000,
  "index_state_path": "index-state.json"
}
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `enable_initial_index` | `true` | Enable or disable initial indexing on startup |
| `progress_interval` | `1000` | Log progress every N files scanned |
| `index_state_path` | `"index-state.json"` | Path to store the index state (relative to `.flashgrep/`) |

## Performance Metrics

After each initial scan, flashgrep logs performance metrics:

```
INFO Initial scan complete: 1500 scanned, 23 added, 5 modified, 2 deleted (took 2.5s, 600.0 files/sec)
```

Metrics include:
- **Files scanned** - Total number of files processed
- **Files added** - New files detected
- **Files modified** - Changed files detected
- **Files deleted** - Removed files detected
- **Duration** - Time taken to complete the scan
- **Files per second** - Processing speed

## Ignore Patterns

The initial scan respects your `.flashgrepignore` file patterns. Files matching ignore patterns are skipped during scanning, just like they are during real-time watching.

Example `.flashgrepignore`:
```
# Build directories
target/
node_modules/
dist/

# IDE files
.idea/
.vscode/

# Logs
*.log
```

## Storage

The index state is stored in `.flashgrep/index-state.json` (or your configured path). This file contains:
- File paths
- File sizes
- Modification timestamps
- Content hashes (first 8KB)

The file is updated atomically to prevent corruption, and old entries are automatically cleaned up (compacted) when files are deleted.

## Disabling Initial Indexing

If you prefer to start watching without an initial scan (for faster startup on large repositories), you can disable it:

```bash
# Edit config.json
echo '{"enable_initial_index": false}' > .flashgrep/config.json

# Or use environment variable
FLASHGREP_ENABLE_INITIAL_INDEX=false flashgrep start
```

Note: Disabling initial indexing means changes made while the watcher was offline won't be detected until those files are modified again.

## Best Practices

1. **Keep ignore patterns up to date** - Exclude large directories (like `node_modules/`, `target/`) to improve scan performance
2. **Monitor progress logs** - For large repositories, watch the progress logs to ensure scanning is proceeding
3. **Regular restarts** - The index state is updated incrementally, so occasional watcher restarts help keep the state fresh
4. **Check metrics** - Review the scan metrics to understand the performance characteristics of your repository

## See Also

- [File Watcher](./file-watcher.md) - Real-time file watching documentation
- [Configuration](./configuration.md) - Full configuration options
- [Troubleshooting](./troubleshooting.md) - Common issues and solutions
