## Why

Users need a quick way to report runtime environment details when troubleshooting or filing issues. The current `--version` output only shows the app version, which is not enough to diagnose platform-specific behavior.

## What Changes

- Add a top-level `version` command (`flashgrep version`) in addition to existing `--version`/`-V` flags.
- Expand version output to include Flashgrep version, operating system, CPU architecture, and related build/runtime metadata useful for support.
- Ensure output is stable and machine-readable enough for copy/paste into bug reports.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `cli-interface`: Extend version behavior to include an explicit `version` command and richer environment details in output.

## Impact

- Affected CLI command parsing and command dispatch logic.
- Affected version output formatting and metadata sourcing (OS/arch/build info).
- No external service or dependency changes expected.
