## Context

Flashgrep already exposes MCP tools and maintains skill guidance in `skills/SKILL.md`, but agents still rely on manual external skill loading. This causes inconsistent startup behavior and frequent fallback to generic grep/glob workflows. The requested change is to make skill injection native to Flashgrep so agents can trigger bootstrap (`flashgrep-init` or `fgrep-boot`) and immediately receive Flashgrep-first operating guidance, including efficient `read_code`/`write_code` usage.

## Goals / Non-Goals

**Goals:**
- Provide a native MCP bootstrap path that returns agent-consumable skill instructions from `skills/SKILL.md`.
- Support init aliases (`flashgrep-init`, `fgrep-boot`) with deterministic response shape.
- Make bootstrap idempotent within a session and return consistent status metadata (`injected`, `already_injected`, `skill_version`, `source_path`).
- Include explicit Flashgrep-first usage guidance in bootstrap payloads so agents prefer indexed tools over generic grep/glob.
- Handle missing/invalid skill content with structured error responses and recovery hints.

**Non-Goals:**
- Enforcing behavior inside third-party agents beyond provided guidance payloads.
- Replacing existing MCP search/read/write tools.
- Building a network-hosted skill registry or remote template system.

## Decisions

### Decision 1: Add a dedicated bootstrap MCP method
Add a new method (for example `bootstrap_skill`) and accept `trigger` values (`flashgrep-init`, `fgrep-boot`) to standardize invocation.

Alternative considered: overloading `initialize` with embedded skill content. Rejected because it couples protocol handshake with large instructional payloads and complicates compatibility.

### Decision 2: Source skill from in-repo `skills/SKILL.md`
Load skill text from `skills/SKILL.md` at runtime and return it in a structured bootstrap response.

Alternative considered: embedding static skill text in code. Rejected because it creates drift from documentation and increases maintenance cost.

### Decision 3: Session-level idempotent injection state
Track whether bootstrap has already been served in the current MCP server session and return `already_injected` on repeated calls unless explicit force refresh is requested.

Alternative considered: always re-send full skill content. Rejected because it wastes tokens and adds repeated prompt overhead.

### Decision 4: Explicit Flashgrep-first policy section in payload
Bootstrap payload includes a concise policy block instructing preference order: `query/files/symbol/read_code/write_code` before fallback tools.

Alternative considered: relying only on generic skill markdown. Rejected because direct policy framing at bootstrap is clearer for agents and reduces ambiguous tool selection.

### Decision 5: Structured error model for skill loading failures
Return machine-readable bootstrap errors (`skill_not_found`, `skill_unreadable`, `invalid_trigger`) with remediation hints.

Alternative considered: plain text failures. Rejected because callers cannot reliably branch behavior.

## Risks / Trade-offs

- [Agents may ignore injected guidance] -> Mitigation: include explicit action-oriented policy and tool preference list in bootstrap payload.
- [Skill markdown can grow too large] -> Mitigation: support compact mode in bootstrap response and include `skill_hash` for cache-aware reuse.
- [Session tracking differs across transport modes] -> Mitigation: keep idempotency state scoped to server process and document behavior.
- [Alias proliferation may confuse callers] -> Mitigation: keep canonical trigger names and return canonicalized value in response.

## Migration Plan

1. Introduce bootstrap handler and response schema in MCP server.
2. Implement skill loader from `skills/SKILL.md` plus hash/version metadata.
3. Add trigger alias handling (`flashgrep-init`, `fgrep-boot`) and idempotent session tracking.
4. Update docs (`skills/SKILL.md`, README MCP section) with bootstrap examples and Flashgrep-first guidance.
5. Add tests for success path, repeated calls, invalid trigger, and missing skill file behavior.

Rollback strategy: disable bootstrap route registration; existing MCP tools continue to function unchanged.

## Open Questions

- Should bootstrap return full markdown by default or a compact policy plus optional full-skill flag?
- Should idempotency be strictly per process, or keyed by client identity if provided?
- Do we also want a CLI helper command that proxies bootstrap over stdio for easier manual testing?
