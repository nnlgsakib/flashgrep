# Flashgrep MCP Skill

Use this skill to run high-speed, index-first code discovery and editing with Flashgrep MCP tools.

## 1) Session Bootstrap

Call once at session start:
- `bootstrap_skill`
- `flashgrep-init` or `flashgrep_init`
- `fgrep-boot` or `fgrep_boot`

Recommended call:

```json
{"name":"flashgrep_init","arguments":{"compact":true}}
```

Expected:
- first call: `status = injected`
- repeat call: `status = already_injected`
- invalid alias/trigger: `error = invalid_trigger`
- default source: `payload_source = embedded`
- optional repo override: set `allow_repo_override = true` (falls back with typed reason if unavailable)

Bootstrap policy contract (machine-readable):
- `policy_metadata.policy_strength = strict`
- `policy_metadata.enforcement_mode = strict`
- `policy_metadata.payload_source` + `policy_metadata.bootstrap_state` are always present
- `policy_metadata.preferred_tools` lists Flashgrep-first routes
- `policy_metadata.fallback_rules` define allowed fallback gates + reason codes
- `policy_metadata.compliance_checks` defines expected compliance behavior

## 2) Primary Tool Selection

Pick the smallest tool that solves the task:

1. `query` / `flashgrep_query`: indexed grep-style search.
2. `glob` / `files` / `list_files` / `flashgrep_glob`: indexed or advanced path discovery.
3. `get_symbol` / `flashgrep_get_symbol`: symbol lookup.
4. `read_code` / `flashgrep_read_code`: budgeted code reads.
5. `get_slice` / `flashgrep_get_slice`: exact line ranges.
6. `write_code` / `flashgrep_write_code`: minimal line-range edits.
7. `stats` / `flashgrep_stats`: index health and readiness.
8. `flashgrep fs` (CLI): deterministic filesystem create/list/stat/copy/move/remove when file lifecycle operations are required.

Hard rule: do NOT use legacy/native search tools unless a fallback gate applies.

Mandatory routing rule:
- Do NOT use native `grep`, `glob`, `read`, `write`, `cat`, or ad-hoc shell scanning while Flashgrep tools are available.
- In agent environments, this explicitly includes host-native tools named like `Read`, `Write`, `Glob`, and `Grep`.
- Use Flashgrep-native MCP routes first: `query`, `files`/`glob`, `get_symbol`, `read_code`, `write_code`.

Allowed fallback gates (must record reason code):
- `flashgrep_index_unavailable`
- `flashgrep_operation_not_supported`
- `flashgrep_tool_runtime_failure`
- `repo_override_unavailable`

Use legacy search tools only when a fallback gate is active:
- `search`
- `search-in-directory`
- `search-with-context`
- `search-by-regex`

## 3) Tool Reference (What + When)

### `query`
Use for indexed text matching with parity options.

Core args:
- `text` (required)
- `mode`: `smart | literal | regex`
- `case_sensitive` (or `regex_flags` in regex mode)
- `include`, `exclude`
- `context`
- `limit`

Patterns:
- literal: `{ "text": "a+b", "mode": "literal" }`
- regex: `{ "text": "fn\\s+\\w+", "mode": "regex", "regex_flags": "i" }`
- scoped: add `include` and `exclude`

### `glob`
Use for deterministic file discovery with one-pass filtering.

Core args:
- `path`, `pattern`
- `include`, `exclude`
- `extensions`
- `max_depth`, `recursive`
- `include_hidden`, `follow_symlinks`
- `sort_by`, `sort_order`
- `offset`, `limit`

Best default for automation:
- `sort_by = path`
- `sort_order = asc`
- explicit `offset` and `limit`

### `get_symbol`
Use for exact symbol-name discovery before reading code.

Arg:
- `symbol_name`

### `read_code`
Use for token-efficient reading. Prefer this over full file dumps.

Modes:
- slice mode: `file_path` (+ `start_line`/`end_line`)
- symbol mode: `symbol_name` (+ `symbol_context_lines`)

Budgets:
- `max_lines`, `max_bytes`, `max_tokens`
- continuation via `continuation_start_line`
- if server returns `payload_too_large`, reduce chunk size and continue
- loop until `continuation.completed = true`

### `get_slice`
Use for exact ranges when you already know file + line bounds.

Args:
- `file_path`, `start_line`, `end_line`

### `write_code`
Use for minimal and safe edits.

Args:
- `file_path`, `start_line`, `end_line`, `replacement`
- optional `precondition` (`expected_file_hash`, `expected_start_line_text`, `expected_end_line_text`)

Behavior:
- precondition mismatch returns conflict (`ok: false`, `error: precondition_failed`)
- oversized replacements return `payload_too_large`; split writes into smaller chunks
- large writes can be continued with `continuation_id`, `chunk_index`, `is_final_chunk`

### `list_files` / `stats`
- `list_files`: full indexed file inventory.
- `stats`: verify index exists/fresh before heavy queries.

### `fs_create` / `fs_read` / `fs_write` / `fs_list` / `fs_stat` / `fs_copy` / `fs_move` / `fs_remove`
Use these MCP tools first for filesystem lifecycle operations.

Recommended operations:
- `fs_create` for file/dir creation (`parents`, `dir`)
- `fs_read` and `fs_write` for file reads/writes (`append`, `overwrite`, `dry_run`)
- `fs_list` and `fs_stat` for deterministic metadata output
- `fs_copy` / `fs_move` with explicit `overwrite`, `recursive`, `dry_run`
- `fs_remove` with explicit `recursive`, `force`, `dry_run`

Typed not-found contract:
- Path-aware MCP tools return `error=not_found` with `reason_code`, `target_kind`, `target_path`
- Branch on reason codes: `file_not_found`, `directory_not_found`, `path_not_found`

## 4) Standard Workflows

### Code Discovery
1. `query` for candidate matches.
2. `get_symbol` if symbol-oriented.
3. `read_code` for bounded context.
4. `get_slice` for exact extraction.

### Deterministic File Expansion
1. `glob` with `pattern` + filters.
2. Always set `sort_by`, `sort_order`, `offset`, `limit`.
3. Feed resulting paths into `query` or `read_code`.

### Targeted Editing
1. Locate with `query`/`get_symbol`.
2. Confirm context with `read_code`.
3. Edit with `write_code` + preconditions where possible.
4. Re-read changed range with `read_code` or `get_slice`.

## 5) Guardrails

- Prefer Flashgrep tools over shell grep/find for repeated work.
- Enforce Flashgrep-first routing; require explicit fallback gate + reason code before native fallback tools.
- Keep outputs bounded (`limit`, `offset`, read budgets).
- Use deterministic sorting for automation (`path`, `asc`).
- If results are empty, verify index with `stats`, then broaden scope.
- Use regex mode only when needed; literal/smart is cheaper and safer.
- For large repos, narrow path scope early (`include`/`exclude`, `max_depth`).
- For large IO, keep request payloads small and expect chunked continuation/retry loops.

## 6) Compliance Recovery

If the agent drifts to native tools:
1. Re-run bootstrap with `{ "force": true, "compact": true }`.
2. Re-check `policy_metadata.policy_strength`, `payload_source`, `bootstrap_state`, and fallback gates.
3. Restart task routing from Flashgrep tools (`query`/`glob`/`read_code`/`write_code`).
4. Keep fallback usage only under a declared gate + reason code.

## 7) Native Tool Ban List

Unless a declared fallback gate is active, treat these as prohibited:

- Native search: `grep`, `rg`, `find`, agent `Grep`
- Native file discovery: shell globbing, agent `Glob`
- Native file IO: `cat`, `sed`, agent `Read`, agent `Write`

If any banned tool is used, immediately recover by re-bootstrap and continue with Flashgrep-native tools.
