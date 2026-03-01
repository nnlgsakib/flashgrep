## Why

Neural retrieval depends on a local model, but users may not know when the model is required or whether it is available. Prompting at index/start time improves discoverability and avoids silent failures while preserving user control over downloads.

## What Changes

- Add model availability checks when indexing starts and when watcher initial indexing starts.
- If the neural model cache is missing, prompt the user to download `BAAI/bge-small-en-v1.5` before continuing.
- If user confirms (`y`), download model assets to `.flashgrep/model-cache/` and continue indexing/start flow.
- If user declines (`n`), continue normal lexical operation without blocking indexing/start.
- Ensure prompts are deterministic and safe for non-interactive environments (fallback behavior documented).

## Capabilities

### New Capabilities
- `neural-model-prompting`: Interactive model download prompt and non-blocking fallback behavior for neural model bootstrapping.

### Modified Capabilities
- `cli-interface`: Index/start command behavior updates to include model availability prompt semantics.
- `indexing-engine`: Startup checks for model cache and continuation behavior when model download is declined.
- `file-watcher`: Initial indexing startup path checks model availability and follows prompt decision.

## Impact

- Affected code: CLI command handlers (`index`, `start`), indexing startup path, watcher initial scan startup flow, model bootstrap helper.
- Data/layout: `.flashgrep/model-cache/` usage is enforced at startup decision points.
- UX/runtime: Users see explicit yes/no prompt when model is missing; declining does not block standard indexing operations.
- Automation: Non-interactive mode needs predictable default behavior when prompting is not possible.
