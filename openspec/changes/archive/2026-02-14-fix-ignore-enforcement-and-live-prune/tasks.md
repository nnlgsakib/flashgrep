## 1. Ignore matching correctness

- [x] 1.1 Add/centralize normalized repo-relative path utility for ignore checks
- [x] 1.2 Apply normalized ignore matching in initial scanner/indexing path
- [x] 1.3 Apply normalized ignore matching in incremental file-event indexing path
- [x] 1.4 Add tests proving `.opencode/` and similar ignored directories are never indexed

## 2. Live ignore reconciliation

- [x] 2.1 Detect `.flashgrepignore` create/modify events in watcher loop
- [x] 2.2 Reload ignore patterns on ignore file change
- [x] 2.3 Compute indexed files that now match ignore rules
- [x] 2.4 Prune newly ignored files from both Tantivy text index and metadata store
- [x] 2.5 Log reconciliation summary (removed/kept counts)

## 3. Metadata prune support

- [x] 3.1 Add bulk metadata prune function for a set of file paths
- [x] 3.2 Ensure prune deletes related file/chunk/symbol records consistently
- [x] 3.3 Make prune idempotent and safe on repeated runs

## 4. Validation and regression coverage

- [x] 4.1 Add integration test: ignored directories do not appear in `files/query/symbol`
- [x] 4.2 Add integration test: updating `.flashgrepignore` removes newly ignored indexed files
- [x] 4.3 Add watcher test: ignore change event triggers reload and reconciliation
- [x] 4.4 Run build/tests and verify no regressions in existing indexing and watcher flows
