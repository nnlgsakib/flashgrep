## 1. Bootstrap Policy Contract

- [x] 1.1 Define bootstrap response policy schema (`preferred_tools`, `fallback_rules`, `policy_strength`, compliance metadata).
- [x] 1.2 Implement policy metadata generation in shared bootstrap builder logic.
- [x] 1.3 Ensure policy metadata remains additive/backward-compatible for existing bootstrap consumers.

## 2. MCP Handler Alignment

- [x] 2.1 Wire bootstrap policy metadata consistently across stdio and TCP MCP paths.
- [x] 2.2 Ensure bootstrap aliases (`bootstrap_skill`, `flashgrep-init`, `flashgrep_init`, `fgrep-boot`, `fgrep_boot`) return equivalent policy semantics.
- [x] 2.3 Add structured fallback-gate reason fields for clients to detect when non-Flashgrep tools are allowed.

## 3. Skill and Documentation Hardening

- [x] 3.1 Refactor `skills/SKILL.md` into strict Flashgrep-first decision flow with explicit fallback gates.
- [x] 3.2 Add concise compliance remediation guidance in `skills/SKILL.md` for agents that drift to native tools.
- [x] 3.3 Update README policy guidance to document enforcement intent, fallback gates, and troubleshooting.

## 4. Validation and Observability

- [x] 4.1 Add tests asserting bootstrap responses include required policy metadata fields.
- [x] 4.2 Add cross-transport tests asserting policy metadata equivalence between stdio and TCP handlers.
- [x] 4.3 Add tests for idempotent bootstrap behavior preserving equivalent policy semantics (`injected` vs `already_injected`).

## 5. Rollout Safety

- [x] 5.1 Add compatibility checks for clients that only consume legacy bootstrap fields.
- [x] 5.2 Document policy strength defaults and downgrade/rollback behavior for interoperability issues.
- [x] 5.3 Run end-to-end bootstrap sanity checks and capture acceptance criteria before release.
