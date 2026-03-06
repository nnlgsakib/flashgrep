# Flashgrep

A high-performance, local code indexing engine designed for LLM coding agents. Flashgrep provides index-first text and structural search for fast repeated queries, deterministic outputs, and automation-friendly workflows.

## Start Here (Install + MCP Init)

If you just want to get running fast:

1. Download the latest binary from the [latest release](https://github.com/nnlgsakib/flashgrep/releases/latest) (or build from source).
2. Open your repository and initialize the local index:

```bash
flashgrep index
```

3. Start the indexer/watcher in the background so the index stays fresh:

```bash
flashgrep start -b
```

4. Configure your MCP client to launch Flashgrep over stdio:

```json
{
  "mcpServers": {
    "flashgrep": {
      "type": "local",
      "command": ["flashgrep", "mcp-stdio"],
      "enabled": true
    }
  }
}
```

5. Run the MCP init/bootstrap command from your client (any alias works):
   - `bootstrap_skill`
   - `flashgrep-init`
   - `flashgrep_init`
   - `fgrep-boot`
   - `fgrep_boot`

This injects policy/tool guidance for the session and prepares Flashgrep-first routing.

## Features

- **Language Agnostic**: Works with any programming language using regex-based heuristics
- **Index-First Performance**: Fast repeated queries after indexing, with incremental updates for changed files
- **Resource Efficient**: Built for low-overhead local operation on medium and large repositories
- **Local Index, Optional Remote LLM**: Core indexing/search stays local; optional neural reranking uses your configured provider API key
- **Token Efficient**: Returns exact code slices, not full files
- **Single Binary CLI**: Distributed as a single executable with local index data in `.flashgrep/`
- **MCP Compatible**: JSON-RPC server for integration with coding agents
- **Lexical Retrieval**: Deterministic indexed search with smart/literal/regex query modes
- **Optional Neural Navigation**: Knowledge-graph-first natural-language navigation using bounded candidate context and provider-assisted reranking
- **Neural-First Option**: For discovery intents, you can run neural retrieval first with deterministic lexical fallback behavior

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/nnlgsakib/flashgrep
cd flashgrep

# Build release binary
cargo build --release

# Install to PATH
cp target/release/flashgrep /usr/local/bin/
```

### Pre-built Binaries

Download pre-built binaries from the [latest release](https://github.com/nnlgsakib/flashgrep/releases/latest) (or browse all [releases](https://github.com/nnlgsakib/flashgrep/releases)).

## Quick Start

```bash
# Navigate to your project
cd /path/to/your/project

# Create initial index
flashgrep index

# Run fast indexed CLI search (grep-like)
flashgrep query "main" --limit 10

# Ask a natural-language question with ranked evidence
flashgrep ask "where is MCP query handled?" --retrieval-mode neural --include "src/**/*.rs" --limit 8

# Start watcher in background (optional)
flashgrep start -b
```

## Usage

### CLI Commands

#### `flashgrep index [PATH]`

Index a repository for searching. If PATH is not provided, indexes the current directory.

```bash
# Index current directory
flashgrep index

# Index specific directory
flashgrep index /path/to/project
```

Features:
- **Incremental indexing**: Only re-indexes changed files
- **Fast**: Indexes 1,500+ files in under 3 seconds
- **Smart filtering**: Ignores `target/`, `node_modules/`, `.git/`, etc.
- **Neural setup prompt**: On first interactive index, prompts for:
  - enable/disable neural navigation
  - provider (`openrouter` / `openai` / `custom`)
  - model
  - base URL
  - API key env var name
  - API key (optional inline)

#### `flashgrep start [PATH]`

Start the daemon with file watcher and MCP server.

```bash
# Start daemon for current directory
flashgrep start

# Start with specific directory
flashgrep start /path/to/project
```

The daemon:
- Watches files for changes and auto-updates index
- Runs MCP server on `localhost:7777`
- Supports graceful shutdown (Ctrl+C)

#### `flashgrep query <TEXT> [PATH]`

Run indexed search with neural-first intent routing when enabled, with deterministic lexical fallback.

```bash
# Find top matches
flashgrep query "fn main" --limit 20

# Script-friendly JSON output
flashgrep query "TODO:" --output json

# Regex mode + path scope + context
flashgrep query "fn\\s+main" --mode regex --include "src/**/*.rs" --context 2

# Literal mode + case-insensitive
flashgrep query "a+b" --mode literal --ignore-case

# Force lexical retrieval
flashgrep query "find authentication middleware" --retrieval-mode lexical --limit 20

# Optional neural-assisted mode (must be enabled/configured)
flashgrep query "find code that sorts names" --retrieval-mode neural --limit 20

# Natural-language function lookup (neural mode)
flashgrep query "find this function \"tokenize\"" --retrieval-mode neural --limit 10

# Force lexical mode explicitly
flashgrep query "tokenize" --retrieval-mode lexical --limit 20
```

#### `flashgrep ask <QUESTION> [PATH]`

Answer a natural-language question by retrieving likely code locations and printing a concise answer with evidence snippets.

```bash
# Natural-language code discovery (neural-first)
flashgrep ask "where is MCP query handled?" --retrieval-mode neural --include "src/**/*.rs" --limit 8

# Force lexical retrieval for deterministic exact-term discovery
flashgrep ask "where is McpServer defined" --retrieval-mode lexical --include "src/**/*.rs" --limit 8

# JSON output for automation
flashgrep ask "how is policy_denied returned" --output json --limit 6
```

Ask behavior:
- Returns a short natural-language answer and ranked evidence snippets.
- Uses the same indexed retrieval engine as `query`; no hallucinated files are invented.
- `--retrieval-mode neural` prefers neural reranking when configured; falls back deterministically when unavailable.
- Returns no-match with a clear hint when evidence is insufficient.

Neural query behavior:
- Uses knowledge-graph/index-first candidate retrieval, then provider-assisted reranking on bounded snippets.
- Recommended discovery order: neural first, then lexical fallback if neural is unavailable or returns no relevant matches.
- Returns `0 result(s)` when no relevant intent match is found (instead of unrelated guesses).
- If provider/API fails, falls back deterministically to lexical retrieval.
- No local model download step is required.

### Neural Navigation Setup

#### Quick setup (interactive)

```bash
flashgrep index --force
```

When prompted, choose provider/model/base URL and set key inline or via env var.

#### Quick setup (environment variable)

OpenRouter:

```bash
# PowerShell
$env:OPENROUTER_API_KEY="your_key_here"
flashgrep query "find vector encoding logic" --retrieval-mode neural --limit 10
```

OpenAI:

```bash
# PowerShell
$env:OPENAI_API_KEY="your_key_here"
flashgrep query "find auth middleware" --retrieval-mode neural --limit 10
```

#### Provider compatibility

Flashgrep neural routing uses an OpenAI-compatible chat completions SDK/client path.
Any provider with OpenAI-compatible endpoints can be configured via:

- `neural.provider.base_url`
- `neural.provider.model`
- `neural.provider.api_key_env` or `neural.provider.api_key`

Default profile:

- provider: `openrouter`
- base_url: `https://openrouter.ai/api/v1`
- model: `arcee-ai/trinity-large-preview:free`
- api_key_env: `OPENROUTER_API_KEY`

You can switch to any free/low-cost OpenAI-compatible provider by changing provider/model/base URL and key settings.

#### `flashgrep files [PATH]`

List indexed files quickly (glob-like exploration without filesystem scans).

```bash
# List indexed files
flashgrep files --limit 100

# Filter file paths
flashgrep files --filter mcp --output json

# Glob-style filtering with deterministic sorting
flashgrep files --pattern "src/**/*.rs" --exclude "**/target/**" --sort-by path --sort-order asc

# Stable pagination window
flashgrep files --pattern "**/*" --offset 200 --limit 100
```

#### `flashgrep fs <SUBCOMMAND> [PATH]`

Run filesystem operations with deterministic behavior for automation scripts.

```bash
# Create file and directory
flashgrep fs create notes/todo.txt --parents
flashgrep fs create build/output --dir --parents

# List and stat with machine-readable JSON
flashgrep fs list src --sort-by path --sort-order asc --offset 0 --limit 50 --output json
flashgrep fs stat src/main.rs --output json

# Copy/move/remove with safety controls
flashgrep fs copy src/main.rs backup/main.rs --overwrite
flashgrep fs move backup/main.rs archive/main.rs --dry-run
flashgrep fs remove archive --recursive --force
```

#### `flashgrep symbol <SYMBOL_NAME> [PATH]`

Find symbol entries from indexed metadata.

```bash
flashgrep symbol McpServer --limit 10
flashgrep symbol main --output json
```

#### `flashgrep slice <FILE_PATH> <START_LINE> <END_LINE> [PATH]`

Extract an exact code range from a file.

```bash
flashgrep slice src/mcp/mod.rs 1 60
flashgrep slice src/search/mod.rs 35 70 --output json
```

#### `flashgrep watchers`

Show active background watcher processes.

```bash
flashgrep watchers
```

### Grep/Glob Replacement Guide

Flashgrep is designed to replace repeated `grep` + filesystem `glob` workflows with deterministic, index-aware operations.

#### Grep-style mappings

- `grep "TODO:" -R src` -> `flashgrep query "TODO:" --include "src/**/*.rs" --limit 200`
- `grep -i "auth" -R .` -> `flashgrep query "auth" --ignore-case --limit 200`
- `grep -E "fn\s+main" -R src` -> `flashgrep query "fn\\s+main" --mode regex --include "src/**/*.rs"`
- `grep -F "a+b" -R src` -> `flashgrep query "a+b" --mode literal --include "src/**/*"`
- `grep -n -C 2 "panic" src/main.rs` -> `flashgrep query "panic" --include "src/main.rs" --context 2`

#### Glob-style mappings

- `glob("src/**/*.rs")` -> `flashgrep files --pattern "src/**/*.rs" --sort-by path --sort-order asc`
- `glob + exclude build dirs` -> `flashgrep files --pattern "**/*" --exclude "**/target/**" --exclude "**/node_modules/**"`
- `glob with extension filter` -> `flashgrep files --pattern "**/*" --ext rs --ext toml`
- `glob pagination/window` -> `flashgrep files --pattern "**/*" --sort-by path --offset 200 --limit 200`

#### Production expectations

- Deterministic output: use explicit `--sort-by`, `--sort-order`, `--offset`, `--limit`.
- Bounded responses: always set `--limit` for scripts/agents.
- Fresh index: run `flashgrep index` first; run watcher (`flashgrep start -b`) for incremental freshness.
- Validation errors: invalid parameter combinations return structured errors (CLI config error or MCP `invalid_params`).
- Large MCP reads/writes: prefer chunked workflows and continuation fields over single oversized payloads.
- Missing paths: path-aware MCP tools return typed not-found diagnostics with `error=not_found`, `reason_code`, `target_kind`, and `target_path`.
- Cross-platform behavior: path filters normalize `/` and `\` separators for deterministic include/exclude matching.
- Filesystem safety: mutating `flashgrep fs` operations support `--dry-run`, and overwrite/delete paths require explicit flags.

### MCP Setup (Stdio)

Use stdio transport for MCP clients that launch local tools as child processes.

1. Build and install `flashgrep`.
2. Index the repository you want to search: `flashgrep index`.
3. Configure your MCP client with the Flashgrep server entry.
4. Start your client and verify Flashgrep tools are available (`ask`, `query`, `glob`, `get_slice`, `read_code`, `write_code`, `batch_write_code`, `fs_create`, `fs_read`, `fs_write`, `fs_list`, `fs_stat`, `fs_copy`, `fs_move`, `fs_remove`, `get_symbol`, `list_files`, `stats`, `bootstrap_skill`, `flashgrep-init`, `fgrep-boot`).
5. Bootstrap is injected automatically during `initialize` using embedded policy guidance.
6. Optionally call `bootstrap_skill` (or alias) to inspect/refresh session policy metadata.

Example MCP config:

```json
{
  "mcpServers": {
    "flashgrep": {
      "type": "local",
      "command": ["flashgrep", "mcp-stdio"],
      "enabled": true,
      "environment": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Notes:
- `RUST_LOG=info` is optional and mainly useful for troubleshooting.
- If your client cannot connect, run `flashgrep index` again and verify `flashgrep stats` works in the same repository.
- For policy routing/debug issues, see `docs/bootstrap-policy-troubleshooting.md`.

Bootstrap example (`tools/call`):

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "flashgrep_init",
    "arguments": {
      "compact": true
    }
  },
  "id": 100
}
```

Bootstrap behavior:
- First call returns `status: injected`
- Repeated call in same server session returns `status: already_injected`
- Embedded payload is default (`payload_source: embedded`) and does not require local skill files
- Optional repository override is opt-in (`allow_repo_override: true`) and falls back deterministically when unreadable
- Policy guidance in response recommends Flashgrep-first tools (`query`, `glob`, `files`, `symbol`, `read_code`, `write_code`, `batch_write_code`) over generic grep/glob fallbacks
- Search routing defaults to neural-first discovery when enabled, with deterministic lexical fallback when neural routing is unavailable or non-relevant
- AI controls are explicit per request (`ai_mode`, `retrieval_mode`, `budget_profile`, `prompt_version`)
- Query payloads expose prompt governance fields (`prompt_id`, `prompt_version`, `prompt_hash`, `policy_rule_hits`)
- Query payloads expose budget telemetry fields (`budget_total`, `tokens_used`, `reduction_applied`, `continuation_id`)

Bootstrap policy metadata:
- `policy_metadata.policy_strength`: enforcement mode (default: `strict`)
- `policy_metadata.enforcement_mode`: strict policy mode for clients
- `policy_metadata.payload_source`: payload origin (`embedded` or `repo_override`)
- `policy_metadata.bootstrap_state`: current session state (`injected` or `already_injected`)
- `policy_metadata.preferred_tool_families`: explicit native Flashgrep routing families
- `policy_metadata.preferred_tools`: Flashgrep-first tool routing groups
- `policy_metadata.search_routing`: neural-first search order and fallback reason contracts
- `policy_metadata.fallback_rules`: allowed fallback gates with typed `reason_code`
- `policy_metadata.compliance_checks`: client-side compliance expectations
- `policy_metadata.prohibited_native_tools`: native/host tools to avoid unless fallback gate is active

Fallback gate defaults:
- `neural_mode_disabled`
- `neural_provider_failure`
- `neural_no_relevant_matches`
- `exact_match_required`
- `query_parse_constraints`
- `flashgrep_index_unavailable`
- `flashgrep_operation_not_supported`
- `flashgrep_tool_runtime_failure`
- `repo_override_unavailable`

Policy enforcement response for ungated fallback routes:

```json
{
  "ok": false,
  "error": "policy_denied",
  "reason_code": "fallback_gate_required",
  "recovery_hint": "Provide fallback_reason_code and fallback_gate, or route via native Flashgrep tools"
}
```

Native-tool routing expectations:
- Agents should avoid host-native `Read`/`Write`/`Glob`/`Grep` and shell `grep`/`cat`/ad-hoc globbing unless a declared fallback gate is active.
- Preferred Flashgrep routes remain `query`, `files`/`glob`, `symbol`/`get_symbol`, `read_code`, `write_code`, `batch_write_code`.
- Fallback tooling (`search`, `search-in-directory`, `search-with-context`, `search-by-regex`) requires explicit `fallback_gate` and `fallback_reason_code`.

AI governance and budget metadata on `query` responses:
- Route fields: `route_state`, `reason_code`, `fallback_gate_id`, `ai_scope`, `budget_profile`.
- Prompt fields: `prompt_governance.prompt_id`, `prompt_governance.prompt_version`, `prompt_governance.prompt_hash`, `prompt_governance.policy_rule_hits`.
- Budget fields: `prompt_budget.budget_total`, `prompt_budget.tokens_used`, `prompt_budget.reduction_applied`, `prompt_budget.continuation_id`.

Compatibility and rollback notes:
- Legacy bootstrap fields (`status`, `canonical_trigger`, `skill_hash`, `skill_version`, `policy`) remain available.
- If a client cannot consume strict metadata, continue reading legacy fields while treating `policy_metadata` as additive.

### Skill Files

Flashgrep provides skill documentation that can be used by any coding agent:

- Primary runtime source: embedded `skills/SKILL.md` payload compiled into the binary
- Canonical editable source: `skills/SKILL.md`
- Optional OpenCode-managed path: `.opencode/skills/flashgrep-mcp/SKILL.md`

`skills/SKILL.md` now uses a compact structured directive language (DSL) to reduce token usage while preserving the same policy behavior.

Example structured directives:

```text
TASK edit_file
FILE src/auth.rs
FIND fn login
REPLACE add rate_limit check
```

Use `skills/SKILL.md` as the canonical authoring source. Runtime bootstrap guidance is embedded at build time, so missing local skill files do not block injection.

### MCP Server API

The MCP server exposes JSON-RPC methods for coding agents. See [MCP Setup (Stdio)](#mcp-setup-stdio) and [Skill Files](#skill-files) for setup and discovery guidance.

**Available Methods:**

#### `bootstrap_skill(trigger?, compact?, force?, allow_repo_override?, repo_override_path?)`

Bootstrap Flashgrep skill guidance into the current MCP session.

Accepted trigger aliases: `bootstrap_skill`, `flashgrep-init`, `flashgrep_init`, `fgrep-boot`, `fgrep_boot`.

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "bootstrap_skill",
    "arguments": {
      "trigger": "flashgrep-init",
      "compact": true
    }
  },
  "id": 0
}
```

#### `glob(...)`

Advanced glob file discovery with composable filters and deterministic sorting.

Supported options include:
- `pattern`, `path`
- `include`, `exclude`
- `extensions`
- `max_depth`, `recursive`, `include_hidden`, `follow_symlinks`
- `case_sensitive`
- `sort_by` (`path|name|modified|size`), `sort_order` (`asc|desc`)
- `limit`

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "glob",
    "arguments": {
      "path": "src",
      "pattern": "**/*.rs",
      "exclude": ["**/target/**", "**/.git/**"],
      "extensions": [".rs"],
      "max_depth": 4,
      "sort_by": "name",
      "sort_order": "asc",
      "limit": 200
    }
  },
  "id": 8
}
```

If the provided root `path` does not exist, glob returns typed not-found diagnostics:

```json
{
  "error": "not_found",
  "reason_code": "directory_not_found",
  "target_kind": "directory",
  "target_path": "..."
}
```

#### `fs_create`, `fs_read`, `fs_write`, `fs_list`, `fs_stat`, `fs_copy`, `fs_move`, `fs_remove`

MCP filesystem lifecycle tools for deterministic automation-safe operations.

- Mutations (`fs_write`, `fs_copy`, `fs_move`, `fs_remove`) support `dry_run`.
- Conflict-sensitive operations support `overwrite`.
- Directory removal supports `recursive` and `force`.
- List/stat responses include stable metadata fields (`path`, `file_type`, `size`, `modified_unix`, `readonly`).
- Missing paths use typed not-found diagnostics (`file_not_found`, `directory_not_found`, `path_not_found`).

#### `query(text, limit)`

Search for text in the indexed codebase.

```json
{
  "jsonrpc": "2.0",
  "method": "query",
  "params": {
    "text": "function main",
    "limit": 10
  },
  "id": 1
}
```

#### `ask(question, limit)`

Natural-language, neural-first codebase Q&A with deterministic lexical fallback.

```json
{
  "jsonrpc": "2.0",
  "method": "ask",
  "params": {
    "question": "where is rpc query handled?",
    "retrieval_mode": "neural",
    "include": ["src/**/*.rs"],
    "limit": 8
  },
  "id": 11
}
```

Response includes:
- `question`, `answer`, and `evidence` snippets
- `route_state` + typed reason metadata
- prompt governance + budget telemetry fields (`prompt_hash`, `policy_rule_hits`, `tokens_used`, `reduction_applied`)

#### `get_slice(file_path, start_line, end_line)`

Retrieve specific lines from a file.

```json
{
  "jsonrpc": "2.0",
  "method": "get_slice",
  "params": {
    "file_path": "src/main.rs",
    "start_line": 1,
    "end_line": 50
  },
  "id": 2
}
```

#### `read_code(...)`

Token-efficient code read for agent workflows. Supports two mutually exclusive modes:

- Slice mode: `file_path` (+ optional `start_line`, `end_line`, `continuation_start_line`)
- Symbol mode: `symbol_name` (+ optional `symbol_context_lines`)

Optional budgets: `max_lines`, `max_bytes`, `max_tokens`.
Optional metadata profile: `metadata_level` (`minimal` or `standard`).

```json
{
  "jsonrpc": "2.0",
  "method": "read_code",
  "params": {
    "file_path": "src/mcp/stdio.rs",
    "start_line": 1,
    "max_lines": 80,
    "metadata_level": "minimal"
  },
  "id": 6
}
```

Response includes deterministic truncation markers and continuation:

- `truncated`: whether output was cut by limits
- `continuation_start_line`: next line to continue from
- `applied_limits`: consumed + configured budgets

Large-IO safety notes:
- Flashgrep enforces MCP payload safety caps to prevent transport disconnects.
- If a request or response is too large, tools return structured `invalid_params` or `payload_too_large` errors.
- For large files, use chunked reads (`max_lines`, `max_bytes`, `continuation_start_line`).
- For full retrieval, loop until `continuation.completed=true` (or `continuation_start_line` is null).

#### `write_code(file_path, start_line, end_line, replacement, precondition?)`

Minimal-diff write that replaces only a target line range. Supports optional optimistic preconditions:

- `expected_file_hash`
- `expected_start_line_text`
- `expected_end_line_text`

On mismatch, returns structured conflict details with `ok: false` and `error: precondition_failed`.

Large-IO safety notes:
- Oversized replacements are rejected with structured `payload_too_large` metadata.
- Retry with smaller replacement chunks to keep the MCP session stable.
- For very large writes, use continuation fields: `continuation_id`, `chunk_index`, and `is_final_chunk`.

```json
{
  "jsonrpc": "2.0",
  "method": "write_code",
  "params": {
    "file_path": "src/example.rs",
    "start_line": 10,
    "end_line": 12,
    "replacement": "updated text",
    "precondition": {
      "expected_start_line_text": "old text"
    }
  },
  "id": 7
}
```

#### `batch_write_code(mode, operations, dry_run?)`

Apply deterministic ordered line-range edits in a single request. Supports two
explicit modes:

- `atomic`: all edits succeed or all edits are rolled back
- `best_effort`: valid edits apply while failed/conflicting edits are reported

Each operation requires: `id`, `file_path`, `start_line`, `end_line`, `replacement`.
Optional per-op preconditions are supported (`expected_file_hash`, line text guards).

```json
{
  "jsonrpc": "2.0",
  "method": "batch_write_code",
  "params": {
    "mode": "atomic",
    "operations": [
      {
        "id": "op1",
        "file_path": "src/example.rs",
        "start_line": 10,
        "end_line": 10,
        "replacement": "let x = 42;"
      }
    ]
  },
  "id": 8
}
```

Response includes per-operation `status` (`applied|failed|conflict|skipped`),
typed `reason_code`, and summary counters.

#### `get_symbol(symbol_name)`

Find all occurrences of a symbol.

```json
{
  "jsonrpc": "2.0",
  "method": "get_symbol",
  "params": {
    "symbol_name": "main"
  },
  "id": 3
}
```

#### `list_files()`

List all indexed files.

```json
{
  "jsonrpc": "2.0",
  "method": "list_files",
  "params": {},
  "id": 4
}
```

#### `stats()`

Get index statistics.

```json
{
  "jsonrpc": "2.0",
  "method": "stats",
  "params": {},
  "id": 5
}
```

## Configuration

### `.flashgrepignore`

Create a `.flashgrepignore` file in your project root to exclude files/directories from indexing. Uses gitignore-style patterns:

```
# Ignore all log files
*.log

# Ignore build directories
build/
dist/

# Ignore specific files
config.local.json

# Re-include specific files
!important.log
```

### Config File

The config is stored in `.flashgrep/config.json`:

```json
{
  "version": "0.1.0",
  "mcp_port": 7777,
  "use_unix_socket": false,
  "socket_path": ".flashgrep/mcp.sock",
  "max_file_size": 2097152,
  "max_chunk_lines": 300,
  "extensions": ["rs", "js", "ts", "py", "go", "json", "md", "yaml", "toml"],
  "ignored_dirs": [".git", "node_modules", "target", "dist", "build", "vendor"],
  "debounce_ms": 500,
  "enable_initial_index": true,
  "progress_interval": 1000,
  "index_state_path": "index-state.json",
  "neural": {
    "enabled": false,
    "initialized": false,
    "provider": {
      "provider": "openrouter",
      "base_url": "https://openrouter.ai/api/v1",
      "model": "arcee-ai/trinity-large-preview:free",
      "api_key_env": "OPENROUTER_API_KEY",
      "api_key": null,
      "timeout_ms": 5000,
      "max_candidates": 24
    }
  }
}
```

Neural mode efficiency rules:
- candidate retrieval stays local knowledge-graph/index-first
- provider calls receive bounded snippet context only
- lexical fallback remains deterministic on provider failures/timeouts

Neural config field notes:

- `neural.enabled`: enable neural retrieval path
- `neural.initialized`: whether setup prompt has already been completed
- `neural.provider.provider`: provider id (`openrouter`, `openai`, or custom)
- `neural.provider.base_url`: OpenAI-compatible API base URL (for OpenRouter use `https://openrouter.ai/api/v1`)
- `neural.provider.model`: chat model id
- `neural.provider.api_key_env`: env var name for key resolution
- `neural.provider.api_key`: optional inline key in config (use carefully)
- `neural.provider.timeout_ms`: provider request timeout
- `neural.provider.max_candidates`: cap on candidate snippets sent for reranking

## Architecture

### Components

- **File Scanner**: Recursively finds indexable files, respects `.flashgrepignore`
- **Chunker**: Splits files into logical chunks (max 300 lines, preserves bracket balance)
- **Symbol Detector**: Regex-based detection of functions, classes, imports, etc.
- **Knowledge Graph Builder**: Builds file/chunk/symbol relationship artifacts for neural candidate routing
- **Tantivy Index**: Full-text search engine with custom ranking
- **SQLite Store**: Metadata storage with connection pooling and batch inserts
- **File Watcher**: Incremental re-indexing with debouncing
- **MCP Server**: JSON-RPC over TCP for agent integration

### Why It Is Faster Than Traditional Grep/Glob Workflows

Flashgrep is often faster than traditional `grep`/`glob` workflows for active development sessions because it is index-first:

- **One-time indexing, many fast reads**: Flashgrep scans/chunks once, then serves queries from Tantivy + SQLite metadata.
- **No full tree scan per query**: traditional grep often re-walks directories and re-reads files every run.
- **Structured metadata paths**: symbol lookup and file listing use indexed tables instead of regex over raw files.
- **Watcher-assisted freshness**: background watcher updates changed files incrementally, avoiding full rebuilds.
- **Deterministic bounded output**: command limits are enforced before render for stable, script-friendly responses.

Use `grep` for tiny one-off folders or ad-hoc exact scans; use Flashgrep when you run many searches per session and want index-backed speed, structure, and deterministic pagination.

### End-to-End Query Flow

1. **Scanner** discovers indexable files and applies ignore rules.
2. **Chunker** splits files into bounded line ranges and computes content hashes.
3. **Symbol Detector** extracts structural entries (function/class/import/etc.).
4. **Knowledge Graph Builder** creates relationship artifacts used for neural candidate expansion.
5. **Tantivy** stores searchable text chunks and ranking fields.
6. **SQLite** stores files/chunks/symbol/graph metadata for lookup/list/stat operations.
7. **CLI/MCP layers** query these stores in read mode and render text/JSON outputs.

### Index Structure

```
.flashgrep/
├── text_index/        # Tantivy full-text index
├── metadata.db        # SQLite database (chunks, symbols, file metadata)
├── config.json        # Configuration
└── vectors/           # Runtime neural/graph auxiliary artifacts
```

## Performance

Example measurements on a typical codebase (1,576 files, ~50k lines). Actual numbers vary by hardware, storage, and repository shape:

- **Initial indexing**: ~2.6 seconds
- **Incremental indexing**: ~0.35 seconds (only changed files)
- **Query response**: <50ms
- **Memory usage**: ~150MB
- **Index size**: ~50MB

## File Support

### Indexed Extensions

- Rust (`.rs`)
- JavaScript/TypeScript (`.js`, `.ts`)
- Python (`.py`)
- Go (`.go`)
- Solidity (`.sol`)
- JSON (`.json`)
- Markdown (`.md`)
- YAML (`.yaml`, `.yml`)
- TOML (`.toml`)

### Ignored Directories

- `.git`
- `node_modules`
- `target`
- `dist`
- `build`
- `vendor`
- `.flashgrep`

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=info cargo run -- index
```

### Documentation Consistency Checks

Before release, verify docs match shipped CLI/MCP behavior:

```bash
# Ensure grep/glob replacement guidance exists
rg "Grep/Glob Replacement Guide" README.md

# Ensure query parity options are documented
rg "--mode regex|--mode literal|--ignore-case|--context" README.md

# Ensure structured skill directives exist and reference primary routes
rg "FORMAT|TOOL_ORDER|WORKFLOW|query|glob|read_code|write_code" skills/SKILL.md
```

### Release Sanity Criteria

Use these pass/fail checks before release:

- `flashgrep stats` returns non-zero indexed file/chunk counts for the target repo.
- `flashgrep query` with parity flags (`--mode`, `--include/--exclude`, `--context`, `--limit`) returns deterministic output shape and no parameter errors.
- `flashgrep files` with deterministic windowing (`--sort-by path --sort-order asc --offset --limit`) returns stable pagination windows.
- MCP `query`/`glob` calls return structured payloads; invalid combinations return `invalid_params`.

### Project Structure

```
src/
├── main.rs           # Entry point
├── lib.rs            # Library root
├── cli/              # Command-line interface
├── config/           # Configuration management
├── db/               # SQLite database layer
├── index/            # File scanning and indexing
├── chunking/         # File chunking logic
├── symbols/          # Symbol detection
├── search/           # Search engine
├── watcher/          # File system watcher
└── mcp/              # MCP server
```

## Troubleshooting

### Index is slow

Use the release build:
```bash
cargo build --release
./target/release/flashgrep index
```

### Out of memory

Reduce cache size in `config.json`:
```json
{
  "max_file_size": 1048576
}
```

### Lock errors

Delete the index and re-index:
```bash
rm -rf .flashgrep
flashgrep index
```


## License

Apache License 2.0 - See LICENSE file for details

## Contributing

Contributions are welcome! Please read CONTRIBUTING.md for guidelines.

## Roadmap

- [ ] Plugin-based language parsers
- [ ] Team shared index
- [ ] Visual graph UI
- [ ] Call graph engine
- [ ] Refactor impact analysis
