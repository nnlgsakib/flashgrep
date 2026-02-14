## Context

Today, `flashgrep start` runs the file watcher in the foreground for one repository path. This blocks the terminal and forces users to keep one terminal per active repository. The requested workflow needs a background mode (`start -b`) and the ability for one Flashgrep installation to manage watchers for multiple repository roots concurrently.

## Goals / Non-Goals

**Goals:**
- Add CLI background mode (`flashgrep start -b`) that detaches watcher execution and returns immediately.
- Support multiple concurrently running watchers, each scoped to a separate repository root.
- Provide lifecycle management per project (start idempotency, status visibility, stop by project).
- Preserve current foreground behavior for users who do not opt into background mode.

**Non-Goals:**
- No change to MCP query/get_slice/get_symbol/list_files/stats protocol semantics.
- No distributed watcher orchestration across machines.
- No requirement to merge indexes from multiple repositories into a single shared index.

## Decisions

- **Decision: Introduce background flag on existing start command**
  - **Rationale:** Keeps command surface simple and backward compatible.
  - **Alternative considered:** New command (`flashgrep daemon start`).
  - **Why not alternative:** More disruptive and unnecessary for this scope.

- **Decision: Use per-project process model with registry metadata**
  - **Rationale:** Simplifies isolation and failure handling; each project watcher can restart/stop independently.
  - **Alternative considered:** One long-running supervisor process managing all projects internally.
  - **Why not alternative:** Higher complexity and migration cost for current architecture.

- **Decision: Make start idempotent per project**
  - **Rationale:** Prevents accidental duplicate watchers and lock contention.
  - **Alternative considered:** Allow duplicate starts and rely on lock errors.
  - **Why not alternative:** Poor UX and noisy failure mode.

- **Decision: Add explicit status/stop behavior by project root**
  - **Rationale:** Multi-project support needs clear operational controls.
  - **Alternative considered:** Global stop only.
  - **Why not alternative:** Users need selective control when multiple repos are active.

## Risks / Trade-offs

- **[Risk]** Stale registry entries after crashes -> **Mitigation:** Validate recorded PID/process existence before reporting watcher as active.
- **[Risk]** Duplicate watchers if project path normalization differs -> **Mitigation:** Canonicalize absolute paths before start/stop/status checks.
- **[Trade-off]** Per-project processes increase process count -> **Mitigation:** Keep each watcher lightweight and clearly visible via status output.
- **[Risk]** Platform-specific detach semantics differ (Windows vs Unix) -> **Mitigation:** Use Rust process APIs with platform-specific spawn configuration tests.

## Migration Plan

1. Extend CLI parsing for `start -b` and project-scoped status/stop behavior.
2. Implement watcher registry and path canonicalization.
3. Add background spawn path and idempotent duplicate-start checks.
4. Update command output/help text and error handling.
5. Validate with single-project foreground flow and multi-project background flow.

Rollback strategy: disable `-b` behavior and revert to existing foreground-only start path.

## Open Questions

- Should background mode write stdout/stderr logs to per-project files by default?
- Should `flashgrep stop` without path stop only current project or all active projects?
