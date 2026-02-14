## 1. CLI background mode

- [x] 1.1 Add `-b`/`--background` flag to `flashgrep start`
- [x] 1.2 Implement detached process spawn path for `start -b`
- [x] 1.3 Preserve existing foreground execution path for `start` without `-b`
- [x] 1.4 Print clear success/failure messages for background start attempts

## 2. Multi-project watcher registry

- [x] 2.1 Add project watcher registry model keyed by canonical repository path
- [x] 2.2 Implement path canonicalization utility used by start/stop/status flows
- [x] 2.3 Prevent duplicate watcher start for an already-active canonical path
- [x] 2.4 Detect and clean stale registry entries when process is not alive

## 3. Project-scoped lifecycle management

- [x] 3.1 Update `stop` behavior to target a specific repository path
- [x] 3.2 Ensure stopping one project does not terminate other project watchers
- [x] 3.3 Add/extend status output to show active watchers per project

## 4. Reliability and validation

- [x] 4.1 Add tests for `start -b` success and spawn failure handling
- [x] 4.2 Add tests for multi-project concurrent watcher management
- [x] 4.3 Add tests for duplicate start idempotency and stale entry cleanup
- [x] 4.4 Validate single-project foreground flow remains unchanged
