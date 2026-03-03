## 1. Define structured skill DSL and parity checklist

- [x] 1.1 Define the compact directive vocabulary (`TASK`, `MODE`, `TOOLS`, `FALLBACK`, `RULE`, `EXAMPLE`, edit blocks with `FILE`/`FIND`/`REPLACE`).
- [x] 1.2 Create a behavior-parity checklist covering tool order, neural-first flow, lexical fallback, fallback reason codes, compliance recovery, and native-tool bans.

## 2. Rewrite skill guidance using compact directives

- [x] 2.1 Rewrite `skills/SKILL.md` into the structured directive format with lower token footprint.
- [x] 2.2 Preserve all existing operational semantics and fallback gate metadata references in the rewritten content.
- [x] 2.3 Add concise structured examples for discovery, exact-match lookup, and targeted editing workflows.

## 3. Update supporting documentation and verify consistency

- [x] 3.1 Update `README.md` sections that describe skill format and routing expectations to match the structured style.
- [x] 3.2 Run a consistency pass to ensure README and `skills/SKILL.md` use the same routing/fallback terminology.
- [x] 3.3 Perform manual validation that the rewritten skill still communicates all required policy constraints while using fewer tokens.
