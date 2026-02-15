## 1. Persisted Index Infrastructure

- [x] 1.1 Define index state data structure (path, size, mtime, content hash)
- [x] 1.2 Implement index state storage (save/load from .flashgrep/index-state.json)
- [x] 1.3 Add index state versioning for future compatibility
- [x] 1.4 Implement index compaction to remove stale entries
- [x] 1.5 Add thread-safe access to index state storage

## 2. Initial Directory Scanning

- [x] 2.1 Create async directory walker function
- [x] 2.2 Integrate .flashgrepignore pattern matching into scanner
- [x] 2.3 Implement file metadata extraction (size, mtime, hash)
- [x] 2.4 Add progress logging every N files (configurable, default 1000)
- [x] 2.5 Handle permission errors gracefully during scan
- [x] 2.6 Handle broken symlinks during scan

## 3. Change Detection Logic

- [x] 3.1 Implement previous index loading on watcher startup
- [x] 3.2 Create comparison function to detect added files (in scan, not in previous index)
- [x] 3.3 Create comparison function to detect modified files (in both, different mtime/hash)
- [x] 3.4 Create comparison function to detect deleted files (in previous index, not in scan)
- [x] 3.5 Implement streaming comparison to reduce memory usage
- [x] 3.6 Add duplicate detection to prevent double events (scan then watcher event)

## 4. Synthetic Event Emission

- [x] 4.1 Create synthetic event structures for create/modify/delete
- [x] 4.2 Integrate synthetic events into existing event pipeline
- [x] 4.3 Ensure synthetic events trigger same indexing as real-time events
- [x] 4.4 Add logging for synthetic events to distinguish from real-time
- [x] 4.5 Handle synthetic events during concurrent real-time events

## 5. Concurrent Watching and Indexing

- [x] 5.1 Spawn initial indexing as background task on watcher start
- [x] 5.2 Ensure file system watcher starts immediately without blocking
- [x] 5.3 Implement proper task coordination (indexing vs real-time events)
- [x] 5.4 Add mechanism to prioritize real-time events during indexing
- [x] 5.5 Handle shutdown gracefully (wait for indexing to complete or interrupt)

## 6. Incremental Index Updates

- [x] 6.1 Implement incremental index writes as files are scanned
- [x] 6.2 Add batching for index writes (every N files or time interval)
- [x] 6.3 Handle partial writes safely (atomic file operations)
- [x] 6.4 Add recovery logic for corrupted index files
- [x] 6.5 Implement index write debouncing to avoid excessive I/O

## 7. Testing

- [x] 7.1 Write unit tests for index state storage operations
- [x] 7.2 Write unit tests for change detection logic
- [x] 7.3 Write unit tests for synthetic event generation
- [x] 7.4 Create integration test for full startup flow
- [x] 7.5 Test concurrent watching and indexing scenarios
- [x] 7.6 Test with large repositories (10k+ files)
- [x] 7.7 Test ignore pattern handling during scan
- [x] 7.8 Test race conditions (file modified during scan)

## 8. Configuration and Polish

- [x] 8.1 Add configuration option to enable/disable initial indexing
- [x] 8.2 Add configuration for progress logging interval
- [x] 8.3 Add configuration for index storage location
- [x] 8.4 Add metrics collection for scan performance
- [x] 8.5 Update watcher startup logging to indicate indexing status
- [x] 8.6 Create documentation for new behavior
- [x] 8.7 Add troubleshooting guide for initial indexing issues
