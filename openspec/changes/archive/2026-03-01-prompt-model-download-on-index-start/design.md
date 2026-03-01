## Context

Flashgrep now supports local neural retrieval features that depend on a cached model in `.flashgrep/model-cache/`. Users frequently trigger indexing or watcher startup before model assets are available, which can cause confusing neural feature behavior later. This change introduces explicit startup-time checks and user prompting while keeping non-neural indexing behavior uninterrupted.

Constraints:
- Existing `flashgrep index` and `flashgrep start` lexical behavior must continue even when users decline model download.
- Prompt behavior must be deterministic and safe in non-interactive contexts.
- Model download target remains `BAAI/bge-small-en-v1.5` in `.flashgrep/model-cache/`.

## Goals / Non-Goals

**Goals:**
- Check model cache presence at indexing start and watcher initial index startup.
- Prompt user to download model when missing.
- If user confirms, bootstrap model and continue startup flow.
- If user declines, continue normal indexing/watcher flow without model.
- Define deterministic default in non-interactive environments.

**Non-Goals:**
- Making model download mandatory for all indexing operations.
- Changing ranking or retrieval logic beyond startup prompt behavior.
- Adding a full model manager command suite in this change.

## Decisions

### Decision: Prompt gating only at startup entry points
- **Choice**: Add check-and-prompt at `index` command start and watcher initial indexing startup.
- **Rationale**: Covers the moments users naturally expect setup guidance without adding repeated prompts in hot paths.
- **Alternative considered**: Prompt on every neural query only.
- **Why not**: Later feedback loop and missed onboarding during index-first workflows.

### Decision: Explicit yes/no prompt with non-blocking decline path
- **Choice**: Prompt with binary choice (`y` download, `n` skip).
- **Rationale**: Predictable UX and deterministic behavior for automation docs.
- **Alternative considered**: Auto-download by default without prompt.
- **Why not**: Unexpected network side effects and poor user control.

### Decision: Non-interactive default is continue without download
- **Choice**: If stdin is non-interactive or prompt cannot be shown, continue indexing/start without download and log guidance.
- **Rationale**: Preserves automation reliability and avoids hangs.
- **Alternative considered**: Fail command in non-interactive mode.
- **Why not**: Breaks existing scripts that do not need neural mode immediately.

## Risks / Trade-offs

- [Users miss model install by answering no] -> Mitigation: emit concise reminder and command/path guidance.
- [Prompt adds startup latency] -> Mitigation: only prompt when cache missing; skip prompt when cache present.
- [Non-interactive ambiguity] -> Mitigation: deterministic fallback policy and explicit log message.
- [Download failures reduce trust] -> Mitigation: actionable retry errors with clear cache path details.

## Migration Plan

1. Add model-cache presence checker for startup flows.
2. Add prompt helper for yes/no download decision.
3. Wire helper into `index` and watcher initial index startup paths.
4. Ensure decline/non-interactive paths continue lexical operations.
5. Update docs and tests for prompt, accept, decline, and non-interactive behavior.

Rollback strategy:
- Remove startup prompt wiring while leaving existing bootstrap utility intact.
- Keep indexing/start behavior fully lexical and uninterrupted.

## Open Questions

- Should watcher startup prompt be suppressible via explicit CLI flag in CI?
- Should we persist a "don't ask again" preference in config for this repository?
