## Context

Flashgrep currently stores neural model assets under each repository's `.flashgrep/model-cache` path. This makes first-run prompts and downloads repeat across projects, even when the same embedding model ID is reused. The existing startup prompt flow in `src/neural/mod.rs` asks whether to download, but it does not offer a storage scope choice.

The requested behavior is to keep local caching available while adding a global cache option driven by `.flashgrep` configuration. This change touches configuration schema, model cache path resolution, and interactive prompt behavior.

## Goals / Non-Goals

**Goals:**
- Add an explicit storage-scope choice (`global` or `local`) at model download time.
- Add config support for a global model path in `.flashgrep/config.json`.
- Resolve model cache location from config and selected scope before download/check logic.
- Reuse already-downloaded global models across repositories when model ID matches.

**Non-Goals:**
- Supporting multiple model IDs in this change.
- Changing embedding provider, download backend, or model manifest format.
- Adding remote/shared network cache management beyond local filesystem paths.

## Decisions

- Introduce a config field for model storage path in `.flashgrep/config.json` (for example `global_model_cache_path`) and treat it as optional. If absent, local behavior remains the default.
- Introduce a model storage scope enum in neural flow (`Global`, `Local`) and thread it through path resolution helpers so cache/manifest checks always operate on a concrete resolved directory.
- Keep the existing yes/no startup question for whether to download, and only ask scope when download is accepted. This limits prompt churn and preserves non-interactive behavior.
- In non-interactive mode, skip scope prompt and use deterministic precedence: explicit environment/config override if present, then local fallback. This preserves automation compatibility.
- Keep model manifest validation unchanged; only the resolved root path changes.

Alternatives considered:
- Environment-variable-only global path control: rejected because user requested `.flashgrep` config ownership.
- Global-only cache replacement: rejected because local-per-project isolation remains a valid workflow.

## Risks / Trade-offs

- [Risk] Prompt complexity may increase friction for first-time users. -> Mitigation: ask scope only after user agrees to download and provide clear defaults.
- [Risk] Invalid configured global path causes confusing failures. -> Mitigation: validate path early and emit actionable error text with fallback guidance.
- [Risk] Different repositories may expect independent model lifecycles. -> Mitigation: preserve local scope and make it selectable at prompt time.
- [Trade-off] Additional configuration and branching adds code complexity. -> Mitigation: centralize path resolution in one helper and reuse across cache check/download/embedding init.
