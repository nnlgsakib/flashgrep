# Data Model: Precise Editing and Patch Reliability

## Entity: EditOperation

- Purpose: Represents one exact replacement request.
- Fields:
  - `id` (string, required): Unique operation identifier.
  - `file_path` (string, required): Target file path.
  - `start_line` (integer >= 1, required): Inclusive start line.
  - `end_line` (integer >= start_line, required): Inclusive end line.
  - `replacement` (string, required): Replacement content for target range.
  - `precondition` (object, optional):
    - `expected_file_hash` (string)
    - `expected_start_line_text` (string)
    - `expected_end_line_text` (string)
  - `dry_run` (boolean, optional, default false): Validate and preview without apply.
- Validation rules:
  - Target file MUST exist.
  - Range MUST be within current file line count.
  - Operation ID MUST be unique inside a batch.

## Entity: BatchEditRequest

- Purpose: Groups operations into one deterministic execution.
- Fields:
  - `mode` (enum, required): `atomic` or `best_effort`.
  - `operations` (array<EditOperation>, required): Ordered operation list.
  - `dry_run` (boolean, optional, default false): Batch-level preview mode.
- Validation rules:
  - `operations` MUST be non-empty.
  - Overlapping ranges in the same file MUST be rejected unless explicitly merged
    by a future policy extension.
  - Duplicate `(file_path, start_line, end_line)` tuples MUST be rejected.

## Entity: EditResult

- Purpose: Reports deterministic outcome for a single operation.
- Fields:
  - `id` (string): Operation ID.
  - `status` (enum): `applied`, `skipped`, `failed`, `conflict`.
  - `reason_code` (string, optional): Typed failure/conflict code.
  - `message` (string, optional): Human-readable context.
  - `file_path` (string)
  - `start_line` (integer)
  - `end_line` (integer)
  - `file_hash_before` (string, optional)
  - `file_hash_after` (string, optional)

## Entity: BatchEditResult

- Purpose: Summarizes and audits a batch execution.
- Fields:
  - `mode` (enum): Echo of request mode.
  - `ok` (boolean): Overall result.
  - `results` (array<EditResult>): Per-operation outcomes in deterministic order.
  - `applied_count` (integer)
  - `failed_count` (integer)
  - `conflict_count` (integer)
  - `skipped_count` (integer)

## Entity: PatchPreview

- Purpose: Provides pre-apply visibility for user/agent confirmation.
- Fields:
  - `operation_id` (string)
  - `target` (file + range)
  - `change_summary` (string)
  - `estimated_lines_removed` (integer)
  - `estimated_lines_added` (integer)

## Relationships

- One `BatchEditRequest` contains many `EditOperation`.
- Each `EditOperation` produces one `EditResult`.
- One `BatchEditRequest` produces one `BatchEditResult`.
- `PatchPreview` can be produced for each `EditOperation` before apply.

## State Transitions

### EditOperation lifecycle

`received` -> `validated` -> (`applied` | `conflict` | `failed` | `skipped`)

### BatchEditRequest lifecycle

`received` -> `preflight_validated` -> (`executing` | `rejected`) -> `completed`

- In `atomic` mode: any operation failure/conflict transitions remaining operations
  to `skipped` and the batch ends with `ok=false`.
- In `best_effort` mode: operations continue in deterministic order after failures,
  and batch `ok` is true only if no `failed` or `conflict` outcomes occur.
