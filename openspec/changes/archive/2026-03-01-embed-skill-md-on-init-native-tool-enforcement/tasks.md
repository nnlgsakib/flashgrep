## 1. Embedded Skill Payload Foundation

- [x] 1.1 Add canonical embedded skill payload source using compile-time embedding (`include_str!`) in bootstrap module
- [x] 1.2 Add payload-source metadata field(s) (`embedded`, `repo_override`) to bootstrap response shape
- [x] 1.3 Ensure bootstrap succeeds when `skills/SKILL.md` is missing or unreadable by defaulting to embedded payload

## 2. Initialization Injection and Idempotency

- [x] 2.1 Wire embedded payload injection into init/bootstrap first-call path
- [x] 2.2 Preserve idempotent session behavior (`injected` on first call, `already_injected` on subsequent calls)
- [x] 2.3 Add deterministic metadata fields for injection state and payload source on every bootstrap response

## 3. Strict Tool Priority Policy Enforcement Metadata

- [x] 3.1 Extend policy metadata to include explicit preferred Flashgrep-native tool families
- [x] 3.2 Add explicit fallback-gate identifiers and typed fallback reasons for native-tool usage
- [x] 3.3 Add enforcement-strength/compliance metadata fields for client-side verification and reporting

## 4. Optional Repo Override Gate

- [x] 4.1 Implement explicit opt-in gate for repository-local skill override (disabled by default)
- [x] 4.2 Add deterministic fallback to embedded payload when override read/validation fails
- [x] 4.3 Emit source/fallback diagnostics metadata for override success/failure cases

## 5. Test Coverage and Regression Safety

- [x] 5.1 Add unit/integration tests for bootstrap success without repository skill file
- [x] 5.2 Add tests verifying policy metadata completeness (tool order, fallback gates, enforcement mode, source)
- [x] 5.3 Add tests verifying idempotent bootstrap semantics and stable compatibility fields across aliases

## 6. Documentation and Operational Guidance

- [x] 6.1 Update `README.md` to document embedded bootstrap default behavior and verification of payload source
- [x] 6.2 Update `skills/SKILL.md` guidance to align with embedded init injection and strict fallback gating
- [x] 6.3 Update troubleshooting docs with steps to diagnose policy drift and bootstrap source state
