## Context

Flashgrep MCP currently returns mixed error envelopes across tool handlers when targets are missing. Some paths return generic `invalid_params`, some return plain text errors, and some cases do not provide explicit not-found typing that agents can branch on. This is most visible in `read_code`/`get_slice` and path-rooted discovery calls where agents need deterministic feedback to decide whether to retry, create a path, or change strategy. In addition, filesystem lifecycle operations are available in CLI but not fully available through MCP tool calls, forcing agents to fall back to native host tools for create/write/list/delete flows.

## Goals / Non-Goals

**Goals:**
- Introduce a consistent not-found error contract for MCP path-based operations.
- Ensure missing file and missing directory conditions are explicitly represented with typed machine-readable fields.
- Add full MCP filesystem tool coverage (create, read, write, list, stat, copy, move, remove) with deterministic operation semantics.
- Preserve backward-compatible MCP response envelope shape while improving diagnostics.
- Add regression tests for not-found behavior in stdio/TCP handlers.


**Non-Goals:**
- Redesigning the full MCP error taxonomy beyond missing-path diagnostics.
- Changing successful response payload shapes for existing tools.
- Introducing interactive recovery prompts in MCP responses.
- Replacing existing CLI filesystem commands; this change mirrors capability into MCP.

## Decisions

### Decision: Add a shared typed not-found payload schema
Use a common not-found structure in tool-level error payloads (e.g., `error: not_found`, `target_kind`, `target_path`, `reason_code`) so agents can parse one format regardless of tool.

Alternatives considered:
- Keep per-tool custom messages: rejected due to inconsistent agent behavior.
- Use only JSON-RPC `error` object: rejected because existing handlers primarily return tool envelope content with `isError`.

### Decision: Normalize missing-path behavior in read and glob entry points
Path-aware handlers (`read_code`, `get_slice`, glob root/path handling) will detect filesystem absence early and return typed not-found payloads instead of generic failures.

Alternatives considered:
- Let lower-level IO exceptions bubble up unchanged: rejected due to nondeterministic formatting.
- Convert all failures to `invalid_params`: rejected because it hides actionable not-found semantics.

### Decision: Preserve compatibility while adding fields
Maintain existing outer MCP response shape (`content` + `isError`) and extend payload body with additive fields so older clients still work and newer agents can route reliably.

Alternatives considered:
- Break to a new envelope format: rejected for migration cost.

### Decision: Add dedicated MCP filesystem tools with deterministic safety controls
Introduce MCP tools for filesystem lifecycle operations rather than forcing agents to combine generic read/write calls. Tool set covers `fs_create`, `fs_read`, `fs_write`, `fs_list`, `fs_stat`, `fs_copy`, `fs_move`, `fs_remove` with explicit flags for overwrite, recursive, dry-run, and force.

Alternatives considered:
- Reuse only `read_code`/`write_code` for all file operations: rejected because directory and metadata operations are not covered.
- Require shell fallback for lifecycle operations: rejected because it undermines deterministic MCP-first workflows.

### Decision: Update skill routing to MCP filesystem tools first
Skill and README guidance will direct agents to use MCP filesystem tools for lifecycle actions and reserve native tool fallback for declared gates.

Alternatives considered:
- Keep existing guidance focused only on search/read/write: rejected because agents still drift to native file operations.

## Risks / Trade-offs

- [Over-normalizing errors may hide nuance] -> Mitigation: keep `reason_code` and include original detail text where useful.
- [Different tools may map path kind differently] -> Mitigation: define strict `target_kind` enum (`file`, `directory`, `path`).
- [Client assumptions on old messages] -> Mitigation: additive fields only; preserve existing top-level error behavior.
- [Filesystem tool expansion increases API surface] -> Mitigation: consistent naming, shared validation helpers, and schema-driven tests.
- [Mutation tools raise safety concerns] -> Mitigation: require explicit overwrite/recursive/force flags and dry-run support.

## Migration Plan

1. Introduce shared helper for not-found payload construction in MCP layer.
2. Apply helper to read/slice and glob path-root validation paths.
3. Add MCP filesystem tools and shared filesystem operation validators/serializers.
4. Add test cases for missing file, missing directory, invalid root path, and filesystem lifecycle operations in stdio/TCP integration flows.
5. Update docs/skill guidance with expected reason codes, filesystem tool usage, and agent branching behavior.

Rollback strategy:
- Revert helper usage and keep prior per-tool formatting while retaining test fixtures for future reintroduction.

## Open Questions

- Should missing file and missing directory use separate reason codes in all handlers or one generalized `path_not_found` with `target_kind`?
- Should tool handlers include optional remediation hints (e.g., "reindex" vs "verify path") in payload metadata?
- Should filesystem MCP tools expose both text and JSON response modes or JSON-only for deterministic automation?
