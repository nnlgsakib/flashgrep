## 1. Shared Not-Found Error Contract

- [x] 1.1 Add a shared MCP helper for typed not-found payload construction (`error`, `reason_code`, `target_kind`, `target_path`)
- [x] 1.2 Define stable reason-code constants for missing file, missing directory, and generic missing path
- [x] 1.3 Ensure helper output remains compatible with existing MCP envelope shape (`content` + `isError`)

## 2. Not-Found Integration in Existing MCP Tools

- [x] 2.1 Integrate typed not-found helper into `read_code` handler path validation
- [x] 2.2 Integrate typed not-found helper into `get_slice` handler path validation
- [x] 2.3 Integrate typed not-found helper into glob root-path validation for missing directory inputs
- [x] 2.4 Normalize stdio and TCP tool-call handlers so not-found payload semantics are equivalent

## 3. MCP Filesystem Tool Surface

- [x] 3.1 Add MCP tool definitions/schemas for filesystem lifecycle operations (`fs_create`, `fs_read`, `fs_write`, `fs_list`, `fs_stat`, `fs_copy`, `fs_move`, `fs_remove`)
- [x] 3.2 Implement stdio MCP handlers for all filesystem lifecycle tools with deterministic structured responses
- [x] 3.3 Implement TCP MCP handlers for all filesystem lifecycle tools with equivalent semantics
- [x] 3.4 Implement mutation safety controls (`overwrite`, `recursive`, `dry_run`, `force`) and deterministic conflict behavior

## 4. Filesystem Not-Found and Validation Semantics

- [x] 4.1 Ensure filesystem MCP tools use shared typed not-found contract for missing sources/targets
- [x] 4.2 Add deterministic validation errors for invalid combinations (e.g., non-recursive remove on non-empty dir)
- [x] 4.3 Ensure list/stat output fields are stable and machine-readable for automation

## 5. Tests

- [x] 5.1 Add unit tests for not-found payload builder fields and reason-code stability
- [x] 5.2 Add stdio MCP tests for missing file and missing directory tool calls
- [x] 5.3 Add TCP MCP tests confirming equivalent not-found error semantics across transport
- [x] 5.4 Add MCP filesystem tool tests for create/list/stat/copy/move/remove success + safety edge cases
- [x] 5.5 Run full Rust test suite and fix regressions

## 6. Documentation and Skill Updates

- [x] 6.1 Update README with MCP filesystem tool mappings, usage examples, and not-found diagnostics contract
- [x] 6.2 Update `skills/SKILL.md` to route file lifecycle actions to MCP filesystem tools before native fallbacks
- [x] 6.3 Update troubleshooting docs with missing-path and filesystem mutation diagnostics guidance
