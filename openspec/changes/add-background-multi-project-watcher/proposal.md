## Why

The current `flashgrep start` command runs in the foreground and blocks a terminal, which makes it awkward for day-to-day use. It also behaves like a single-project process, while users often need active file watching for multiple repositories at the same time.

## What Changes

- Add a background mode (`flashgrep start -b`) that starts the watcher as a detached process and returns control to the terminal.
- Add multi-project watcher management so one installed Flashgrep can manage watchers for multiple repository roots concurrently.
- Add lifecycle commands to inspect and stop watchers per project (while preserving current behavior for foreground mode).
- Add clear status and error messages for duplicate starts, missing watchers, and background launch failures.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `cli-interface`: Extend CLI requirements to support background start mode and multi-project watcher management commands.
- `file-watcher`: Extend watcher requirements to support concurrent project watchers with per-project lifecycle management.

## Impact

- **Modified code areas**: `src/cli/`, `src/watcher/`, and supporting state/process management modules.
- **User-facing behavior**: New `-b` background option and multi-project management semantics.
- **Operational impact**: Users can keep watchers active for multiple repositories without keeping multiple terminals open.
