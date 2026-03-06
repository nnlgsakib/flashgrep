# Contract: Precise Editing and Batch Editing

## Scope

This contract defines user-facing behavior for precise write operations and
deterministic batch editing in Flashgrep MCP tooling.

## Tool Contract: `write_code`

### Required inputs

- `file_path`
- `start_line`
- `end_line`
- `replacement`

### Optional inputs

- `precondition.expected_file_hash`
- `precondition.expected_start_line_text`
- `precondition.expected_end_line_text`
- `continuation_id`, `chunk_index`, `is_final_chunk` (for chunked writes)

### Behavioral guarantees

- Range replacement is exact: only requested line range is replaced.
- Operation is deterministic for identical inputs and file state.
- If preconditions fail, tool returns `ok=false` and `error=precondition_failed`.
- Oversized replacement returns typed payload-too-large error.

### Required outputs

- `ok` (boolean)
- `file_path`, `start_line`, `end_line`
- `file_hash_before`, `file_hash_after` (on success)
- `error` and structured conflict details (on failure/conflict)

## Tool Contract: `batch_write_code`

### Required inputs

- `operations` (ordered list of edit operations)
- `mode` (`atomic` or `best_effort`)

### Optional inputs

- `dry_run` (validate/preflight without apply)

### Preflight requirements

- Reject duplicate operation IDs.
- Reject invalid ranges.
- Reject overlapping operations in same file.
- Validate preconditions before apply.

### Behavioral guarantees

- Deterministic execution order equals request order.
- `atomic`: no edits are committed if any operation fails preflight or apply.
- `best_effort`: valid edits apply; failed/conflicting operations are reported.
- Per-operation result is always returned with typed status and reason code.

### Required outputs

- `ok` (boolean)
- `mode` (echoed)
- `results[]` with one entry per operation:
  - `id`
  - `status` (`applied|failed|conflict|skipped`)
  - `reason_code` (optional)
  - `file_path`, `start_line`, `end_line`
  - `file_hash_before`, `file_hash_after` (when available)
- Summary counters: `applied_count`, `failed_count`, `conflict_count`,
  `skipped_count`

## Error Taxonomy (typed)

- `precondition_failed`
- `invalid_range`
- `file_not_found`
- `overlapping_operations`
- `duplicate_operation_id`
- `invalid_continuation_state`
- `payload_too_large`
- `internal_error`

## Documentation and Skill Parity Requirements

- README editing examples MUST match this contract.
- `skills/SKILL.md` edit workflows MUST include precondition-safe write patterns.
- Release checks MUST verify example parity with returned status fields and reason
  codes.
