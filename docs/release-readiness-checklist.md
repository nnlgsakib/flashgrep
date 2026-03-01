# Release Readiness Checklist

Use this checklist before declaring Flashgrep production-ready as a grep/glob replacement.

## Compatibility and Behavior

- [ ] Query fixed-string, literal, and regex modes behave as documented.
- [ ] Query exit status contract is validated (`0` match, `1` no match, non-zero failure).
- [ ] Include/exclude matching behaves deterministically with `/` and `\\` separators.
- [ ] Files command supports deterministic windows with `sort + offset + limit`.

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
- [ ] Parity matrix updated (`docs/grep-glob-production-parity-matrix.md`).
