## Why

Currently, the flashgrep file watcher only detects changes that occur while it's actively running. When the watcher is started, it has no knowledge of what files were added or deleted while it was offline. Users need the watcher to perform an initial index of all files on startup so it can detect and report changes that happened during downtime, providing a complete view of the repository state.

## What Changes

- **Initial indexing on watcher start**: When the file watcher starts, it will scan and index all files in the repository
- **Non-blocking operation**: Initial indexing runs asynchronously without blocking new file watching
- **Change detection from baseline**: Compare current file state with previous index to detect additions, modifications, and deletions
- **Background processing**: Indexing happens in the background while the watcher is already monitoring for new changes
- **Progress reporting**: Optional progress indicators for large repositories during initial scan

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `file-watcher`: Add initial indexing behavior when watcher starts, with non-blocking async processing and change detection from previous state

## Impact

- Changes to watcher startup sequence in the file watcher subsystem
- May increase startup time for large repositories (mitigated by async processing)
- Provides more accurate change detection across watcher restarts
- Improves user experience by ensuring no changes are missed between sessions
