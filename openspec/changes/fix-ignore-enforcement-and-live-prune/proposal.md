## Why

`.flashgrepignore` patterns are not being enforced reliably in all indexing paths, so ignored directories like `.opencode/` still appear in results. Users also need ignore changes to take effect immediately: when a new ignore pattern is added, already-indexed files under that path should be dropped automatically.

## What Changes

- Fix ignore matching so `.flashgrepignore` directory patterns are consistently applied during indexing and watcher-driven updates.
- Add live ignore reconciliation so watcher/index workflows remove already-indexed files that become ignored after `.flashgrepignore` changes.
- Add pruning behavior to remove ignored files from both text index and metadata store.
- Add clear diagnostics for ignore reload and prune counts so users can verify behavior.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `indexing-engine`: Strengthen `.flashgrepignore` enforcement and ensure ignored paths are never indexed.
- `file-watcher`: React to `.flashgrepignore` updates and trigger live prune/reconciliation.
- `metadata-store`: Support bulk removal of file/chunk/symbol records for newly ignored paths.

## Impact

- **Affected code**: ignore parsing/matching, scanner/index loop, watcher event handling, DB prune operations.
- **User-visible behavior**: ignored directories/files stop appearing in `query`, `files`, and `symbol` results, including after dynamic ignore changes.
- **No breaking API changes**: existing CLI/MCP commands remain the same; behavior becomes more correct.
