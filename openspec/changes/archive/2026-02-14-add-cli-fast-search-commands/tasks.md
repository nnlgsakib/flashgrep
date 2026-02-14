## 1. Add CLI command surface for fast indexed search

- [x] 1.1 Add new CLI subcommands for `query`, `files`, `symbol`, and `slice`
- [x] 1.2 Add core arguments for each command (search text, limits, paths, line ranges)
- [x] 1.3 Add `--json` (or equivalent) output mode option for script-friendly output
- [x] 1.4 Update command help text with concise usage examples

## 2. Wire commands to existing flashgrep core

- [x] 2.1 Connect `query` command to indexed text search path
- [x] 2.2 Connect `files` command to indexed file listing with optional filtering
- [x] 2.3 Connect `symbol` command to symbol metadata lookup
- [x] 2.4 Connect `slice` command to file line-range extraction

## 3. Result shaping and output consistency

- [x] 3.1 Enforce deterministic ordering and command limits before rendering output
- [x] 3.2 Implement consistent plain-text output fields across commands
- [x] 3.3 Implement consistent JSON field naming across commands
- [x] 3.4 Add actionable errors for missing index and invalid arguments

## 4. Validation and quality checks

- [x] 4.1 Add tests for each new CLI command success path
- [x] 4.2 Add tests for invalid argument and missing-index failure modes
- [x] 4.3 Add tests that verify stable ordering/limit behavior
- [x] 4.4 Run build and test suite; ensure no regressions in existing commands

## 5. Skill documentation updates

- [x] 5.1 Update `skills/SKILL.md` to document new CLI search commands (`query`, `files`, `symbol`, `slice`)
- [x] 5.2 Add command usage examples and JSON output examples to `skills/SKILL.md`

## 6. README and architecture docs updates

- [x] 6.1 Update `README.md` with new CLI command docs (`query`, `files`, `symbol`, `slice`, `watchers`)
- [x] 6.2 Document why Flashgrep is more efficient than traditional `grep`/`glob` in `README.md`
- [x] 6.3 Add architecture and efficiency deep-dive doc at `docs/flashgrep-vs-grep-glob.md`
