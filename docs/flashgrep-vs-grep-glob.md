# Flashgrep vs Traditional Grep/Glob

## Purpose

This document explains why Flashgrep is often more efficient than traditional `grep`/`glob` workflows for ongoing development, and how the architecture enables that speed.

## Short Answer

- `grep`/`glob`: fast for one-off scans, but usually rescans filesystem and file contents each run.
- Flashgrep: pays indexing cost once, then answers repeated searches from local indexes with low latency.

## Where Flashgrep Gains Efficiency

1. **Indexed execution path**
   - Text queries run against Tantivy index segments.
   - File/symbol operations read SQLite metadata tables.
   - Most commands avoid full repository scans on each invocation.

2. **Incremental updates**
   - File watcher tracks changes and re-indexes only modified files.
   - Index freshness improves without full reindex cycles.

3. **Structured retrieval modes**
   - `query`: ranked text matches.
   - `files`: index-backed file listing.
   - `symbol`: metadata-level symbol lookup.
   - `slice`: direct line-range extraction.

4. **Script-oriented output**
   - `--output json` allows deterministic parsing.
   - Bounded limits reduce shell noise and post-processing overhead.

## Architecture Overview

### Data Plane

- **Scanner**: discovers candidate files.
- **Ignore engine**: applies `.flashgrepignore` and built-in excludes.
- **Chunker**: creates bounded line chunks with hashes.
- **Symbol detector**: extracts structural symbols.
- **Tantivy index**: full-text ranking and retrieval.
- **SQLite store**: file/chunk/symbol metadata and stats.

### Serving Plane

- **CLI commands** for terminal-first usage (`query`, `files`, `symbol`, `slice`, `stats`).
- **MCP servers** (`mcp`, `mcp-stdio`) for coding-agent integration.
- Both layers use the same indexed core.

## Operational Model

1. Run `flashgrep index` for initial build.
2. Optionally run `flashgrep start -b` for background incremental updates.
3. Use CLI or MCP queries repeatedly with low latency.

## Practical Guidance

- Prefer Flashgrep for active coding sessions where you run many searches.
- Keep watcher running for near-real-time index freshness.
- Use `grep` for tiny ad hoc tasks where index setup is not worth it.

## Limitations and Trade-offs

- Initial index build is an upfront cost.
- Results depend on index freshness if watcher is not running.
- Regex semantics differ from direct grep in some edge cases.

## Suggested Benchmark Method

Compare repeated queries over the same repository:

1. Warm repository state.
2. Run 20 representative text/symbol/file queries with Flashgrep.
3. Run equivalent `grep`/`glob` commands.
4. Compare p50/p95 latency and total wall-clock time.

For medium/large repositories and repeated lookup workflows, Flashgrep typically provides better end-to-end productivity.
