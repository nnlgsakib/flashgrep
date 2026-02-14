## Context

Flashgrep already has a fast indexing and search core, but day-to-day terminal workflows still default to traditional `grep`/`glob` style scanning. Users want direct CLI commands that expose the same indexed speed and structure-aware behavior without needing MCP integration.

## Goals / Non-Goals

**Goals:**
- Add CLI-first fast search commands for common workflows: text search, file listing, symbol lookup, and code slicing.
- Keep command output suitable for both human reading and shell piping.
- Reuse existing index/search internals so commands remain low-latency on large repositories.

**Non-Goals:**
- No replacement of existing MCP methods.
- No dependency on external search tools (`grep`, `glob`) for core execution.
- No semantic/AST parser introduction in this scope.

## Decisions

- **Decision: Add dedicated CLI subcommands instead of overloading existing commands**
  - **Rationale:** Clear discoverability and stable UX for script usage.
  - **Alternative considered:** Add flags to existing `stats`/`mcp` commands.
  - **Why not alternative:** Harder to learn and less explicit for grep-like workflows.

- **Decision: Map commands directly to existing core operations**
  - **Rationale:** Minimizes implementation risk and preserves performance characteristics.
  - **Alternative considered:** Build a separate CLI search pipeline.
  - **Why not alternative:** Duplicate logic and increased maintenance overhead.

- **Decision: Include output-mode controls (default text + structured option)**
  - **Rationale:** Supports both interactive and automation use cases.
  - **Alternative considered:** Human-readable output only.
  - **Why not alternative:** Limits shell and CI integration value.

- **Decision: Keep limit/filter controls aligned with existing search constraints**
  - **Rationale:** Predictable behavior and bounded response size.
  - **Alternative considered:** Unbounded result output by default.
  - **Why not alternative:** Risk of noisy output and slower terminal UX.

## Risks / Trade-offs

- **[Risk]** Users may expect exact `grep` flag parity -> **Mitigation:** Document command differences and provide practical examples.
- **[Risk]** Large output volumes can reduce perceived speed -> **Mitigation:** Default limits and optional stricter output filtering.
- **[Trade-off]** Additional CLI surface area increases maintenance -> **Mitigation:** Reuse shared core functions and centralized formatting utilities.
- **[Risk]** Output schema drift between CLI and MCP -> **Mitigation:** Define explicit result-shaping requirements in specs.

## Migration Plan

1. Add CLI command definitions and argument parsing.
2. Wire commands to existing search/index metadata calls.
3. Add output formatting options and command help examples.
4. Add integration tests for common grep/glob replacement flows.
5. Validate performance and command ergonomics on medium/large repos.

Rollback: hide/disable new commands and retain existing CLI behavior only.

## Open Questions

- Should JSON output be enabled for all new search commands or only selected ones?
- Should default file listing include ignored/internal directories, or stay index-scoped only?
