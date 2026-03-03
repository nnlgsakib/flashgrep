## Why

`skills/SKILL.md` has grown long and verbose, which increases token usage for agent bootstrapping and repeated planning calls. We need a compact, structured instruction language that preserves current behavior and routing policy while reducing token overhead.

## What Changes

- Rewrite `skills/SKILL.md` into a concise command-style DSL (task-oriented blocks such as `TASK`, `FILE`, `FIND`, `REPLACE`, and compact routing directives).
- Preserve all current behavior contracts: Flashgrep-first routing, neural-first discovery with lexical fallback, fallback-gate reason codes, tool ordering, and safety constraints.
- Keep bootstrap aliases and metadata expectations intact while shortening instruction phrasing.
- Add canonical examples for code discovery, exact lookup, and targeted editing in the structured format.

## Capabilities

### New Capabilities
- `structured-skill-language`: Defines a compact, machine-friendly instruction grammar for `skills/SKILL.md` that reduces token usage without changing behavior.

### Modified Capabilities
- `ai-agent-documentation`: Skill guidance format changes from verbose prose to structured DSL while preserving all routing and compliance requirements.

## Impact

- Affected files: `skills/SKILL.md`, related references in `README.md`, and bootstrap/skill docs that describe expected instruction format.
- No runtime API break expected; impact is documentation/bootstrap payload shape and token efficiency.
- Agent usability impact: lower token consumption and faster parseability while maintaining same policy semantics.
