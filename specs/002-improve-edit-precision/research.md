# Research: Precise Editing and Patch Reliability

## Decision 1: Enforce optimistic preconditions on all write paths

- Decision: Require precondition checks (`expected_file_hash` and line text guards)
  before applying single or batch writes.
- Rationale: This prevents stale overwrites and wrong-target writes when files
  changed after read/preview.
- Alternatives considered:
  - Blind line-range replacement (rejected: high overwrite risk)
  - Timestamp-only checks (rejected: weak conflict signal)

## Decision 2: Use strict read-verify-write-verify workflow

- Decision: Standard workflow uses read/preview, precondition validation, apply,
  then post-apply verification read.
- Rationale: Detects duplicate insertion, offset drift, and partial write issues
  early, and gives deterministic operator feedback.
- Alternatives considered:
  - Write-only flow (rejected: unsafe in concurrent edits)
  - Exit-code-only validation (rejected: does not prove semantic correctness)

## Decision 3: Add explicit batch modes (`atomic`, `best_effort`)

- Decision: Support two explicit batch execution modes; default to `atomic`.
- Rationale: `atomic` is safest for automation, while `best_effort` supports large
  maintenance edits where partial success is acceptable.
- Alternatives considered:
  - Always atomic (rejected: too rigid for large migrations)
  - Always best-effort (rejected: inconsistent workspace state)

## Decision 4: Preflight overlap/conflict detection in batch requests

- Decision: Validate duplicate/overlapping ranges before execution; reject
  ambiguous edit sets deterministically.
- Rationale: Overlapping writes are a primary source of duplicate or clobbered
  lines in multi-op runs.
- Alternatives considered:
  - Last-write-wins conflict policy (rejected: silent data loss)
  - Runtime-only conflict detection (rejected: late failures)

## Decision 5: Deterministic ordered execution and typed per-op results

- Decision: Execute operations in stable order and return per-operation status
  with machine-readable reason codes and hashes.
- Rationale: Reproducibility is required for CI/automation and enables selective
  retry of failed edits.
- Alternatives considered:
  - Parallel unordered execution (rejected: non-deterministic outcomes)
  - Batch-level success flag only (rejected: insufficient diagnostics)

## Decision 6: Preserve file invariants during patch application

- Decision: Preserve newline style and trailing-newline behavior and avoid
  whole-file normalization in edit operations.
- Rationale: Invariant drift can create phantom diffs and perceived duplicates.
- Alternatives considered:
  - Normalize entire file every write (rejected: noisy and risky)
  - Delegate newline handling to platform defaults (rejected: cross-platform drift)

## Decision 7: Keep edits minimal and contiguous

- Decision: Apply the smallest practical contiguous ranges for each write.
- Rationale: Smaller blast radius reduces accidental changes and merge conflicts.
- Alternatives considered:
  - Whole-file rewrite strategy (rejected: high collateral risk)

## Decision 8: Treat docs and skill guidance as release-gated artifacts

- Decision: Require docs and `skills/SKILL.md` updates in the same change set as
  behavior changes, with parity checks before release.
- Rationale: Prevents drift between implementation and user/agent behavior.
- Alternatives considered:
  - Post-release docs updates (rejected: stale guidance window)
  - Manual-only governance reminders (rejected: weak enforcement)
