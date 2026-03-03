## Context

Flashgrep currently runs lexical/programmatic navigation with deterministic, index-backed behavior. A prior change removed neural routing to simplify the stack. This change re-introduces optional neural navigation, but with an explicit architecture constraint: retrieval quality and efficiency are driven by indexing and a knowledge graph, while the model is a bounded assistive layer for intent interpretation and concise answer shaping.

The feature must support low-cost/free model providers with OpenRouter as default, allow user-supplied API keys and model selection, and remain efficient on large repositories.

## Goals / Non-Goals

**Goals:**
- Add optional neural navigation for natural-language code discovery.
- Build a knowledge-graph-aware neural index so candidate retrieval remains local/index-first.
- Support provider/model/base URL/API key configuration with sensible defaults:
  - Provider: `openrouter`
  - Base URL: `https://openrouter.ai/api/v1/chat/completions`
  - Model: `arcee-ai/trinity-large-preview:free`
- Prompt at index time to enable neural navigation and persist opt-in config.
- Enforce efficiency guardrails (bounded candidates, bounded prompt size, caching).

**Non-Goals:**
- Replacing lexical navigation as default behavior.
- Streaming whole-file/codebase content to remote models.
- Building a cloud-only index; all primary retrieval remains local.
- Introducing non-deterministic fallback behavior that bypasses policy gates.

## Decisions

1. Two-stage retrieval pipeline (index-first)
   - Stage A: local retrieval over lexical index + semantic metadata + knowledge graph neighborhoods to produce bounded candidates.
   - Stage B: optional model call to disambiguate intent and rank/summarize only bounded candidates.
   - Rationale: preserves speed/cost constraints and makes model usage token-efficient.
   - Alternatives:
     - Model-first retrieval over raw repository text: rejected (costly, non-deterministic, poor scale).
     - Pure local embeddings without model assist: rejected for ambiguous natural-language intents.

2. Knowledge graph structure
   - Nodes: files, symbols, chunks, imports, references.
   - Edges: contains, calls, imports, references, same-module/co-change heuristics.
   - Graph persisted in local metadata store with indexed lookups by node and relation.
   - Rationale: enables efficient intent expansion without full-context prompting.

3. Provider abstraction with OpenRouter defaults
   - Add provider config object: `provider`, `base_url`, `model`, `api_key_env`, `api_key`, timeout/retry/token limits.
   - Default provider profile matches OpenRouter request contract and free model.
   - API key resolution order: explicit config key -> env var -> prompt/setup guidance.
   - Rationale: user flexibility with safe defaults and deterministic behavior.

4. Index-time neural enablement flow
   - `flashgrep index` prompts whether to enable neural navigation when config is unset.
   - If enabled, writes neural config and schedules semantic/graph artifact generation.
   - If disabled, lexical indexing proceeds unchanged.
   - Non-interactive runs use configured default and never block.
   - Rationale: explicit user consent, predictable automation behavior.

5. Efficiency constraints as first-class rules
   - Fixed max candidate set before model call.
   - Prompt compaction from chunk snippets + graph evidence only.
   - Query/result cache keyed by index revision + normalized intent.
   - Strict timeout and fallback to lexical results on model failures.
   - Rationale: keeps latency/tokens bounded while preserving utility.

## Risks / Trade-offs

- [Risk] Graph/semantic indexing overhead increases index time and storage. -> Mitigation: make neural artifacts opt-in, incremental, and separately measurable.
- [Risk] Provider outages or key issues degrade neural UX. -> Mitigation: deterministic fallback to lexical mode and explicit diagnostics.
- [Risk] Prompt leakage of too much content increases token cost. -> Mitigation: snippet/candidate budget caps and hard token limits.
- [Trade-off] Additional config complexity for providers/models. -> Mitigation: OpenRouter defaults + guided setup + documented env-based key management.

## Migration Plan

1. Extend config schema for neural provider and enablement fields.
2. Add CLI setup/index-time prompt logic and non-interactive defaults.
3. Implement semantic+graph artifact generation in indexing pipeline.
4. Add neural query mode over two-stage retrieval pipeline with strict budgets.
5. Update MCP policy/docs to describe optional neural routing and efficiency guardrails.
6. Rollout strategy: default remains lexical; neural enabled only via explicit opt-in.
7. Rollback: disable neural mode in config and ignore neural artifacts while keeping lexical search intact.

## Open Questions

- Should provider keys be persisted encrypted in config, or only referenced via env var by default?
- Which embedding model (local vs provider-hosted) should back graph candidate expansion for best speed/quality tradeoff?
- Should model-assisted ranking return scored evidence/explanations by default or behind a verbose flag only?
