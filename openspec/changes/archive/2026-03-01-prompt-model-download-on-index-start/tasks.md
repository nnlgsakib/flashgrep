## 1. Prompt and Decision Flow

- [x] 1.1 Add a reusable startup model-cache check helper for index/start initialization paths.
- [x] 1.2 Implement interactive yes/no prompt when `BAAI/bge-small-en-v1.5` cache is missing.
- [x] 1.3 Implement decision handling: `y` downloads model and continues, `n` skips download and continues normally.

## 2. CLI and Watcher Integration

- [x] 2.1 Wire prompt flow into `flashgrep index` startup before indexing work begins.
- [x] 2.2 Wire prompt flow into watcher startup before initial indexing begins.
- [x] 2.3 Ensure startup behavior remains deterministic in non-interactive environments (no blocking prompts).

## 3. Bootstrap, Logging, and UX

- [x] 3.1 Reuse existing model bootstrap logic to download into `.flashgrep/model-cache/` after user confirmation.
- [x] 3.2 Add clear informational logs/messages for accepted, declined, skipped, and failed download paths.
- [x] 3.3 Ensure decline path preserves lexical indexing and daemon startup behavior without errors.

## 4. Validation and Documentation

- [x] 4.1 Add tests for prompt accepted, declined, and non-interactive fallback behavior.
- [x] 4.2 Add tests for index/start integration paths to confirm normal continuation when user selects `n`.
- [x] 4.3 Update CLI/docs with prompt behavior and model cache guidance.
