## Why

Flashgrep currently supports programmatic/lexical navigation only. Users want an optional natural-language navigation experience (for example, "find code for sorting names" or "which files contain auth logic") while keeping results deterministic, efficient, and index-first.

## What Changes

- Add an optional neural navigation mode that is powered by repository indexing + a knowledge graph, not by pure LLM reasoning.
- Add a neural index pipeline that builds semantic metadata/graph artifacts for intent-style retrieval with low token usage.
- Add provider/model configuration (default provider: OpenRouter) including API key, base URL, and model selection.
- Add index-time prompt/flow to enable or disable neural navigation and persist configuration choices.
- Add strict retrieval policy where the model is used for query interpretation and lightweight ranking/formatting, while core candidate retrieval remains index-backed.
- Add efficient request shaping and caching limits to reduce token and latency costs.

## Capabilities

### New Capabilities
- `neural-navigation`: Optional natural-language code navigation over indexed repository data.
- `knowledge-graph-indexing`: Graph and semantic index construction for efficient neural-assisted retrieval.
- `llm-provider-config`: Provider/model/key/base-url configuration for neural navigation execution.

### Modified Capabilities
- `search-engine`: Add index-backed neural-assisted retrieval pipeline while preserving lexical behavior.
- `cli-search-commands`: Add user-facing neural navigation query mode and related flags/options.
- `cli-interface`: Add index-time enablement prompt and configuration UX for neural navigation.
- `model-storage-scope`: Extend config structure to include provider/model/base URL/key wiring for neural mode.
- `agent-tool-priority-policy`: Define deterministic routing for optional neural navigation with explicit fallback to lexical/programmatic paths.
- `ai-agent-documentation`: Document neural navigation usage, efficiency guardrails, and provider configuration defaults.

## Impact

- Affected code areas: `src/index/`, `src/search/`, `src/cli/`, `src/config/`, `src/mcp/`, and docs in `README.md` + `skills/SKILL.md`.
- New external integration: OpenRouter-compatible chat completions API (default model `arcee-ai/trinity-large-preview:free`) with pluggable provider endpoint.
- Configuration impact: new provider/model/base URL/API key settings and neural enablement flag.
- Performance/cost impact: additional indexing artifacts and runtime path; mitigated with graph-first retrieval, candidate bounding, caching, and token budgets.
