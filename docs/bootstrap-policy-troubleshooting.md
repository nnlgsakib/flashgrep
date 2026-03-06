# Bootstrap Policy Troubleshooting

Use this guide when agents drift to native tools or bootstrap metadata does not appear as expected.

## 1) Verify init-time policy injection

On MCP stdio, `initialize` includes a `bootstrap` payload.

Check:

- `bootstrap.status` is `injected` (first call) or `already_injected` (repeat)
- `bootstrap.payload_source` is `embedded` by default
- `bootstrap.policy_metadata.policy_strength` is `strict`

## 2) Force-refresh session policy

Call:

```json
{
  "name": "bootstrap_skill",
  "arguments": { "compact": true, "force": true }
}
```

Then verify:

- `policy_metadata.bootstrap_state`
- `policy_metadata.payload_source`
- `policy_metadata.fallback_rules`

## 3) Diagnose payload source

Default behavior:

- Source is embedded at build time (`payload_source = embedded`)

Optional repository override:

- Set `allow_repo_override = true`
- Optional `repo_override_path` may be provided

If override cannot be read, bootstrap falls back deterministically:

- `payload_source = embedded`
- `fallback_gate = repo_override_read_failed`
- `fallback_reason_code = repo_override_unavailable`

## 4) Diagnose fallback usage

Native fallback is allowed only under explicit gates:

- `index_unavailable` (`flashgrep_index_unavailable`)
- `unsupported_operation` (`flashgrep_operation_not_supported`)
- `tool_runtime_failure` (`flashgrep_tool_runtime_failure`)
- `repo_override_read_failed` (`repo_override_unavailable`)

If no gate applies, re-route to Flashgrep-native tools.

Also verify ban-list metadata:

- `policy_metadata.prohibited_native_tools.search`
- `policy_metadata.prohibited_native_tools.discovery`
- `policy_metadata.prohibited_native_tools.file_io`

These should map to tools your agent must avoid unless a declared fallback gate is active.

## 5) Recovery steps for policy drift

1. Re-run bootstrap with force.
2. Confirm strict metadata (`policy_strength`, `enforcement_mode`).
3. Confirm `payload_source` and `bootstrap_state`.
4. Resume Flashgrep-first routing (`query`, `files/glob`, `symbol`, `read_code`, `write_code`, `batch_write_code`).

## 6) Missing path and filesystem diagnostics

For path-aware MCP operations (`read_code`, `get_slice`, `glob`, `fs_*`), verify typed diagnostics:

- `error = not_found`
- `reason_code` in (`file_not_found`, `directory_not_found`, `path_not_found`)
- `target_kind` in (`file`, `directory`, `path`)
- `target_path` with the missing path

For filesystem mutations (`fs_write`, `fs_copy`, `fs_move`, `fs_remove`), verify safety semantics:

- conflict without overwrite: `error = conflict`, `reason_code = destination_exists`
- invalid combination: `error = invalid_params` with deterministic `reason_code`
- `dry_run = true` returns operation plan without mutation

For batch edits (`batch_write_code`), verify deterministic edit diagnostics:

- explicit `mode` is required for non-default semantics
- per-operation `status` is one of `applied|failed|conflict|skipped`
- typed `reason_code` appears for failed/conflicting/skipped operations
- summary counters (`applied_count`, `failed_count`, `conflict_count`, `skipped_count`) are present

For fallback search tooling (`search*`), verify policy enforcement diagnostics:

- missing gate metadata returns `error = policy_denied`
- denied payload includes typed `reason_code` (`fallback_gate_required`, `unsupported_fallback_reason_code`, or `fallback_gate_mismatch`)
- drift mismatches return `reason_code = policy_state_mismatch` with recovery hint
