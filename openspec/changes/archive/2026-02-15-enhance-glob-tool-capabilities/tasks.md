## 1. Schema and Input Contract

- [x] 1.1 Extend glob MCP input schema to support advanced options (`include`, `exclude`, `extensions`, `max_depth`, `include_hidden`, `follow_symlinks`, `case_sensitive`, `sort_by`, `sort_order`, `limit`).
- [x] 1.2 Implement robust validation for new options, including incompatible combinations and invalid values, with structured parameter errors.
- [x] 1.3 Preserve existing defaults and behavior when advanced options are omitted.

## 2. Core Glob Engine Enhancements

- [x] 2.1 Implement traversal-time include/exclude filtering and depth-bounded traversal (`max_depth`) to avoid post-filter-only scanning.
- [x] 2.2 Implement extension filtering with normalized matching behavior and tests for both dotted and non-dotted forms if supported.
- [x] 2.3 Implement hidden-file/symlink handling controls (`include_hidden`, `follow_symlinks`) with safe defaults.
- [x] 2.4 Add deterministic sorting (`sort_by`, `sort_order`) with stable tie-breaking.
- [x] 2.5 Apply result limiting efficiently (`limit`) without unnecessary extra traversal where possible.

## 3. MCP Integration and Compatibility

- [x] 3.1 Wire enhanced glob options through MCP handlers while keeping existing glob method compatibility.
- [x] 3.2 Ensure response structure remains compatible for existing callers and includes deterministic output ordering.
- [x] 3.3 Add parity checks for advanced glob behavior in both stdio and TCP MCP paths if both expose glob.

## 4. Documentation Updates

- [x] 4.1 Update README glob documentation with advanced parameter reference and practical examples.
- [x] 4.2 Update `skills/SKILL.md` with one-pass glob filtering strategies for efficient agent workflows.
- [x] 4.3 Document recommended patterns for large-repo performance (exclude + depth + limit + deterministic sort).

## 5. Verification and Performance

- [x] 5.1 Add tests for extension filtering, include/exclude precedence, and depth bounds.
- [x] 5.2 Add tests for sorting/limit determinism and backward-compatible default behavior.
- [x] 5.3 Add tests for invalid option combinations returning structured errors.
- [x] 5.4 Run full test suite and confirm no regressions.
