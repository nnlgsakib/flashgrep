## Why

Flashgrep currently includes neural/semantic search and AI navigation routing behavior that adds model/runtime complexity the team no longer wants to maintain. We need to remove AI-based navigation completely and return to a deterministic lexical-first search stack while keeping all non-AI capabilities stable.

## What Changes

- **BREAKING** Remove neural/semantic/hybrid retrieval support from CLI and MCP query surfaces.
- **BREAKING** Remove neural model bootstrap, download prompt, cache lifecycle, and vector-backed retrieval requirements.
- Remove AI agent neural-first routing policy requirements and update tool-priority policy to programmatic-only routing.
- Update agent/README guidance so discovery workflows rely on lexical/programmatic search behavior only.
- Preserve existing non-AI features (indexing, lexical search, filesystem tools, symbol lookup, metadata store, watcher, and deterministic output behavior).

## Capabilities

### New Capabilities
- _None._

### Modified Capabilities
- `neural-command-support`: Remove requirements for model download/cache lifecycle and vector-backed semantic retrieval.
- `neural-model-prompting`: Remove startup neural model prompt and non-interactive neural fallback requirements.
- `agent-neural-routing-policy`: Remove neural-first routing and typed neural fallback requirement set.
- `agent-tool-priority-policy`: Replace neural-first discovery routing with deterministic Flashgrep programmatic routing only.
- `ai-agent-documentation`: Remove neural-first guidance and update docs to lexical/programmatic discovery workflows.
- `cli-search-commands`: Remove semantic/hybrid query mode requirements while preserving lexical query behavior and compatibility flags.
- `search-engine`: Remove semantic and hybrid query requirements; keep lexical ranking, formatting, and performance contracts.

## Impact

- Affected code is expected across `src/neural/`, query/search command handling, MCP tool schemas/metadata, bootstrap policy payloads, and related docs/tests.
- CLI and MCP clients relying on `semantic`/`hybrid` retrieval modes will need to migrate to lexical/programmatic search.
- Build/runtime footprint should simplify by removing neural model dependencies and cache workflows.
