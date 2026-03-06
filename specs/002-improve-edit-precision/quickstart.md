# Quickstart: Validate Precise and Batch Editing

## 1) Prepare

```bash
cargo test
```

Confirm baseline tests are green before feature implementation.

## 2) Validate precise single edit safety

1. Create a fixture file with repeated similar lines.
2. Read the target range using Flashgrep read workflow.
3. Apply `write_code` with preconditions set.
4. Re-read the file and confirm:
   - only target range changed,
   - no duplicated lines,
   - no adjacent line overwrite.

Expected result: exact replacement with deterministic hash change.

## 3) Validate conflict behavior

1. Read a range and capture precondition values.
2. Modify the same file externally.
3. Attempt original write again.

Expected result: operation rejected with typed conflict (`precondition_failed`) and
no file mutation from rejected request.

## 4) Validate batch editing (`atomic` mode)

1. Prepare a batch with at least 3 operations.
2. Make one operation intentionally invalid.
3. Execute batch in `atomic` mode.

Expected result:
- batch fails predictably,
- no operation is committed,
- result includes per-operation statuses and reason codes.

## 5) Validate batch editing (`best_effort` mode)

1. Re-run same batch in `best_effort` mode.

Expected result:
- valid operations apply once,
- invalid/conflicting operations are reported,
- final result summary counts are correct and deterministic.

## 6) Validate docs and skill parity

1. Confirm README examples match current write/batch contract fields.
2. Confirm `skills/SKILL.md` includes precondition-safe edit workflow.
3. Run release/readiness checks relevant to docs consistency.

Expected result: guidance and behavior are aligned; no stale examples remain.

## Validation Record

- Date: 2026-03-06
- `cargo test`: pass
- `cargo clippy --all-targets -- -D warnings`: pass
- Batch edit behavior verified for `atomic`, `best_effort`, overlap rejection,
  dry-run no-mutation, and deterministic status reporting.
