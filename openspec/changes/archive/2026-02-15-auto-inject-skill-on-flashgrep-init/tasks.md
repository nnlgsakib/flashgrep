## 1. Bootstrap API Surface

- [x] 1.1 Add a native MCP bootstrap method (for example `bootstrap_skill`) that accepts trigger input and returns structured bootstrap payloads.
- [x] 1.2 Implement trigger normalization/validation for `flashgrep-init` and `fgrep-boot`, and return `invalid_trigger` for unsupported values.
- [x] 1.3 Add bootstrap response schema fields for status (`injected`/`already_injected`), canonical trigger, source path, and skill hash/version metadata.

## 2. Skill Loading and Injection Behavior

- [x] 2.1 Implement runtime loading of `skills/SKILL.md` as the authoritative injection source.
- [x] 2.2 Implement structured bootstrap errors for skill load failures (`skill_not_found`, `skill_unreadable`) with remediation details.
- [x] 2.3 Add session-level idempotency tracking so repeated bootstrap calls return `already_injected` without resending unnecessary payload content.
- [x] 2.4 Include explicit Flashgrep-first policy guidance in bootstrap payloads, prioritizing `query`, `files`, `symbol`, `read_code`, and `write_code` over generic grep/glob fallback.

## 3. MCP Integration and Compatibility

- [x] 3.1 Wire bootstrap handling into MCP request routing for stdio (and other supported transport paths if shared handler exists).
- [x] 3.2 Ensure existing MCP methods (`query`, `get_slice`, `get_symbol`, `list_files`, `stats`, `read_code`, `write_code`) remain behavior-compatible after bootstrap integration.
- [x] 3.3 Add optional compact bootstrap mode and/or hash-based cache hint behavior if needed to control token overhead on large skill payloads.

## 4. Documentation Updates

- [x] 4.1 Update `skills/SKILL.md` with native bootstrap usage (`flashgrep-init`, `fgrep-boot`) and expected bootstrap response behavior.
- [x] 4.2 Update README MCP setup/docs to describe no-manual-skill bootstrap flow and Flashgrep-first tool selection guidance.
- [x] 4.3 Document idempotent reinvocation behavior and troubleshooting for bootstrap failures.

## 5. Verification

- [x] 5.1 Add tests for bootstrap success path, including skill payload metadata and policy guidance fields.
- [x] 5.2 Add tests for alias trigger handling and invalid trigger rejection.
- [x] 5.3 Add tests for repeated bootstrap calls returning `already_injected` semantics.
- [x] 5.4 Add tests for missing/unreadable `skills/SKILL.md` error behavior.
- [x] 5.5 Run full test suite and validate no regressions in existing MCP tool behavior.
