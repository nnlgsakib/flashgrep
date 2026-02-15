## 1. Parity Contract and Option Model

- [x] 1.1 Define shared query option model covering regex/literal mode, case handling, path include/exclude, context, and output bounds.
- [x] 1.2 Define shared glob option model covering include/exclude, extensions, hidden/symlink controls, depth/recursion, sorting, and limit/window behavior.
- [x] 1.3 Add validation rules and structured error shapes for invalid or conflicting parity options across CLI and MCP.

## 2. Query Engine and CLI Parity

- [x] 2.1 Implement grep-compatible query behaviors in the indexed search pipeline (regex/literal selection, case behavior, path scoping).
- [x] 2.2 Implement context-line rendering and deterministic bounded output metadata for query responses.
- [x] 2.3 Extend `flashgrep query` CLI options and help text to expose parity features while preserving backward-compatible defaults.

## 3. Glob Discovery and CLI Parity

- [x] 3.1 Extend glob/file discovery execution with one-pass combined filtering and traversal-time pruning.
- [x] 3.2 Implement deterministic sorting and bounded windows for stable automation output on large repositories.
- [x] 3.3 Extend `flashgrep files`/glob-facing CLI behaviors and output modes to surface replacement-grade options consistently.

## 4. MCP Contract Alignment

- [x] 4.1 Expose query parity parameters in MCP tool contracts and wire them to shared option models.
- [x] 4.2 Expose glob replacement-grade parameters in MCP tool contracts and enforce structured parameter validation.
- [x] 4.3 Add MCP integration tests validating deterministic outputs and parity behavior consistency with CLI equivalents.

## 5. Documentation and Skill Optimization

- [x] 5.1 Update `README.md` with grep/glob migration mappings, production usage examples, and replacement guidance.
- [x] 5.2 Refactor `skills/SKILL.md` into a compact, token-efficient decision flow while preserving Flashgrep-first tool ordering.
- [x] 5.3 Add documentation checks/review steps to ensure README + skill guidance matches shipped CLI/MCP behavior.

## 6. Verification and Release Readiness

- [x] 6.1 Add regression and performance tests for parity options, deterministic ordering, and bounded outputs.
- [x] 6.2 Validate backward compatibility for existing query/glob calls without advanced options.
- [x] 6.3 Run end-to-end sanity checks on medium/large repositories and capture pass/fail criteria before release.
