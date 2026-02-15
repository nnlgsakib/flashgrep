## Context

The flashgrep file watcher currently starts monitoring immediately when launched, but it lacks awareness of the repository state from before it was started. This means any files added, modified, or deleted while the watcher was offline go undetected. The system already has an indexing engine (as seen in `openspec/specs/indexing-engine/spec.md`) that can scan and index files, but it's not being invoked during watcher startup.

The current watcher startup flow:
1. Initialize file system watcher
2. Start monitoring for real-time changes
3. Log "File watcher started" message

This leaves a gap where changes that occurred between watcher sessions are invisible to the system.

## Goals / Non-Goals

**Goals:**
- Perform a complete repository scan when the file watcher starts
- Detect and report all changes (additions, modifications, deletions) that occurred while watcher was offline
- Ensure initial indexing doesn't block real-time file watching (files changed during indexing should still be detected)
- Maintain a baseline/index state that persists between watcher sessions for comparison
- Provide progress indication for large repository scans
- Minimize performance impact on the system during initial indexing

**Non-Goals:**
- Real-time progress UI with percentage bars (logging is sufficient)
- Parallel scanning of multiple directories (sequential is fine for now)
- Resumable indexing on crash (if watcher crashes, next start does full scan)
- Historical change tracking beyond the last known state
- Differential/indexed scanning (always do full directory walk)

## Decisions

**Decision 1: Async Concurrent Processing**
- **Choice**: Start the file system watcher immediately, then begin initial indexing in a background task concurrently
- **Rationale**: This ensures no file events are missed during the initial scan. Files modified during indexing will trigger watcher events and be re-indexed normally.
- **Alternative**: Block until indexing completes - rejected because it would miss changes during the scan period

**Decision 2: Persisted Index State**
- **Choice**: Store the last known index state (file paths + modification times + content hashes) in a persisted location (e.g., `.flashgrep/index-state.json`)
- **Rationale**: Enables comparison between current state and previous state to detect what changed while offline
- **Alternative**: Store in memory only - rejected because it wouldn't survive watcher restarts

**Decision 3: Three-Way Change Detection**
- **Choice**: Compare three states: previous persisted index, current scan results, and active watcher events
  - Files in current scan but not in previous index = Added while offline
  - Files in both with different mod times/hashes = Modified while offline
  - Files in previous index but not in current scan = Deleted while offline
- **Rationale**: Provides complete picture of all offline changes
- **Alternative**: Only track new files - rejected because modifications and deletions are equally important

**Decision 4: Event Emission for Offline Changes**
- **Choice**: Emit synthetic file change events for offline changes so they go through the same processing pipeline as real-time changes
- **Rationale**: Keeps the system simple - one code path handles all changes
- **Alternative**: Special handling for offline changes - rejected as adds complexity

**Decision 5: Incremental Index Updates**
- **Choice**: Update the persisted index incrementally as files are scanned, not atomically at the end
- **Rationale**: If the watcher crashes during initial indexing, we still have partial state for next comparison
- **Alternative**: Atomic write at completion - rejected as risks losing all progress on crash

## Risks / Trade-offs

**Risk 1: Race Conditions Between Scanning and Watcher Events**
- **Concern**: A file might be scanned, then modified, then the watcher event arrives. We might emit a duplicate "modified" event.
- **Mitigation**: Use file modification timestamps and content hashes to detect duplicates. If a file's hash matches what we just scanned, skip emitting the synthetic event.

**Risk 2: High Memory Usage for Large Repositories**
- **Concern**: Loading entire previous index into memory for comparison could be expensive
- **Mitigation**: Stream the comparison - read previous index entries one by one, check against current scan, don't hold everything in memory

**Risk 3: Performance Impact on Large Repositories**
- **Concern**: Full directory walk on startup could be slow for repositories with 100k+ files
- **Mitigation**: 
  - Use efficient directory walking (parallel where possible)
  - Respect .flashgrepignore patterns to skip ignored files
  - Add configurable concurrency limit
  - Log progress so users know it's working

**Risk 4: Storage Growth from Persisted Index**
- **Concern**: The persisted index file could grow large over time
- **Mitigation**: Periodically compact the index (remove entries for deleted files), store only essential metadata (path, mtime, size, hash)

**Risk 5: Consistency with Ignore Patterns**
- **Concern**: If .flashgrepignore changes between sessions, the comparison might be wrong
- **Mitigation**: Always apply current ignore patterns during scan. If ignore file changed, the reconciliation logic already handles pruning ignored files.

## Migration Plan

**Phase 1: Core Implementation**
1. Add persisted index storage mechanism
2. Implement initial directory scanning on watcher start
3. Add comparison logic to detect offline changes
4. Emit synthetic events for offline changes

**Phase 2: Optimization**
1. Add streaming comparison to reduce memory usage
2. Implement incremental index updates
3. Add progress logging
4. Optimize for large repositories

**Phase 3: Polish**
1. Add configuration options (enable/disable initial index, concurrency limits)
2. Add metrics/logging for scan performance
3. Handle edge cases (permissions, broken symlinks, etc.)

**Rollback Strategy:**
- Add configuration option to disable initial indexing (defaults to enabled)
- If issues arise, users can disable the feature via config
- The change is additive - doesn't break existing functionality
