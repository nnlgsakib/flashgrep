## 1. Config and provider foundation

- [x] 1.1 Extend `.flashgrep` config schema with neural enablement and provider fields (`provider`, `base_url`, `model`, API key source).
- [x] 1.2 Implement deterministic API key resolution order (config key -> env var) and actionable validation errors.
- [x] 1.3 Add default OpenRouter profile (`https://openrouter.ai/api/v1/chat/completions`, `arcee-ai/trinity-large-preview:free`).

## 2. Indexing and knowledge graph artifacts

- [x] 2.1 Add optional neural artifact generation in indexing pipeline gated by neural enablement.
- [x] 2.2 Implement knowledge graph node/edge persistence for files, chunks, symbols, and references.
- [x] 2.3 Implement incremental graph updates and deterministic graph revision tracking for cache keys.

## 3. Neural-assisted query pipeline

- [x] 3.1 Add neural query mode in search engine and CLI query parsing while preserving lexical default behavior.
- [x] 3.2 Implement two-stage retrieval: index/graph candidate selection first, provider inference second on bounded context.
- [x] 3.3 Add deterministic fallback to lexical/index-backed results on provider timeout/failure with typed diagnostics.

## 4. Index-time UX and policy updates

- [x] 4.1 Add interactive `flashgrep index` prompt to enable neural navigation when neural config is unset.
- [x] 4.2 Add non-interactive deterministic behavior (no prompts) using configured defaults.
- [x] 4.3 Update MCP routing policy metadata and skill guidance for optional neural mode with index-first and fallback gates.

## 5. Efficiency, validation, and docs

- [x] 5.1 Add token/latency guardrails (candidate caps, prompt compaction, timeout/retry bounds, cache usage).
- [x] 5.2 Add tests for provider config, enablement flow, graph indexing behavior, and neural query fallback behavior.
- [x] 5.3 Update README and `skills/SKILL.md` with neural setup, provider defaults, and efficiency guidance.
