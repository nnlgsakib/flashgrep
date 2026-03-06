# Release Readiness Checklist

Use this checklist before declaring Flashgrep production-ready as a grep/glob replacement.

## Compatibility and Behavior

- [ ] Query fixed-string, literal, and regex modes behave as documented.
- [ ] Query exit status contract is validated (`0` match, `1` no match, non-zero failure).
- [ ] Include/exclude matching behaves deterministically with `/` and `\\` separators.
- [ ] Files command supports deterministic windows with `sort + offset + limit`.
- [ ] `write_code` exact-range replacement validated with precondition conflicts.
- [ ] `batch_write_code` validates deterministic ordering and mode semantics (`atomic`/`best_effort`).
- [ ] Ungated fallback search routes return typed `policy_denied` diagnostics.
- [ ] Valid fallback gate + reason pairs are admitted and audited.
- [ ] AI discovery requests require explicit `ai_mode` and emit typed route metadata.
- [ ] Prompt governance denials are typed (`prompt_policy_violation`, `prompt_policy_escalation_required`, `prompt_version_unsupported`).
- [ ] Query responses include `prompt_hash`, `policy_rule_hits`, and budget telemetry (`tokens_used`, `reduction_applied`).

## Filesystem Operations

- [ ] `flashgrep fs create/list/stat/copy/move/remove` CLI paths validated.
- [ ] `copy`/`move` conflict behavior requires explicit `--overwrite`.
- [ ] Destructive operations validated with `--dry-run` before execution.
- [ ] Remove behavior validated for file, empty dir, and recursive dir modes.

## Cross-Platform Matrix

- [ ] Linux validation pass.
- [ ] macOS validation pass.
- [ ] Windows validation pass.

## Performance and Stability

- [ ] Indexed query p95 latency measured on representative repository.
- [ ] Large result windows complete with no gaps/duplicates.
- [ ] CLI and MCP payload bounds validated under stress cases.

## Documentation and Skill

- [ ] README includes grep/glob mapping + filesystem examples.
- [ ] `skills/SKILL.md` keeps Flashgrep-first routing with explicit fallback gates.
- [ ] README and skill examples include `batch_write_code` fields and typed status semantics.
- [ ] README and skill examples include `policy_denied`, `fallback_gate`, and `fallback_reason_code` semantics.
- [ ] README and `skills/SKILL.md` include `ai_mode`, `budget_profile`, `prompt_version`, `prompt_hash`, and `policy_rule_hits` guidance.
- [ ] Parity matrix updated (`docs/grep-glob-production-parity-matrix.md`).
