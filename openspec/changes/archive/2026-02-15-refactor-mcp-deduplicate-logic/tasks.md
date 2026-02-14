## 1. Shared MCP Handler Extraction

- [x] 1.1 Identify duplicated bootstrap and alias-normalization logic in `src/mcp/mod.rs` and `src/mcp/stdio.rs`.
- [x] 1.2 Extract shared transport-agnostic helper(s) for bootstrap trigger normalization, validation, and response payload generation.
- [x] 1.3 Rewire both TCP and stdio request paths to use the shared bootstrap helper(s) without changing public method names.

## 2. Tool Definition and Alias Consistency

- [x] 2.1 Consolidate bootstrap/tool alias definitions so one canonical source drives both listing and invocation behavior.
- [x] 2.2 Remove or simplify repetitive hard-coded MCP tool list fragments that duplicate existing definitions.
- [x] 2.3 Ensure alias set remains synchronized across tool discovery and method dispatch (`bootstrap_skill`, `flashgrep-init`, `flashgrep_init`, `fgrep-boot`, `fgrep_boot`).

## 3. Behavioral Parity Verification

- [x] 3.1 Add/adjust tests to validate bootstrap alias normalization consistency across stdio and TCP handlers.
- [x] 3.2 Add/adjust tests to verify bootstrap statuses (`injected`, `already_injected`) and typed errors (`invalid_trigger`, `skill_not_found`, `skill_unreadable`) remain stable.
- [x] 3.3 Add/adjust tests ensuring existing core methods (`query`, `get_slice`, `get_symbol`, `list_files`, `stats`, `read_code`, `write_code`) preserve expected behavior after refactor.

## 4. Cleanup and Regression Validation

- [x] 4.1 Remove nonsensical/redundant MCP logic paths identified during extraction while keeping functionality intact.
- [x] 4.2 Run full test suite and confirm no regressions in MCP behavior.
- [x] 4.3 Perform quick code-level sanity review of `src/mcp/` for maintainability and no remaining obvious duplicate hot spots introduced by this change.
