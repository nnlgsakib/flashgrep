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

## 2) Primary Tool Selection

Pick the smallest tool that solves the task:

1. `query` / `flashgrep_query`: indexed grep-style search.
2. `glob` / `files` / `list_files` / `flashgrep_glob`: indexed or advanced path discovery.
3. `get_symbol` / `flashgrep_get_symbol`: symbol lookup.
4. `read_code` / `flashgrep_read_code`: budgeted code reads.
5. `get_slice` / `flashgrep_get_slice`: exact line ranges.
6. `write_code` / `flashgrep_write_code`: minimal line-range edits.
7. `stats` / `flashgrep_stats`: index health and readiness.

Use legacy search tools only if needed for compatibility:
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
- Keep outputs bounded (`limit`, `offset`, read budgets).
- Use deterministic sorting for automation (`path`, `asc`).
- If results are empty, verify index with `stats`, then broaden scope.
- Use regex mode only when needed; literal/smart is cheaper and safer.
- For large repos, narrow path scope early (`include`/`exclude`, `max_depth`).
- For large IO, keep request payloads small and expect chunked continuation/retry loops.
