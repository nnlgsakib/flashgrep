## Why

Model assets are currently stored per project, which causes repeated downloads across repositories and unnecessary setup time. We need a standard way to reuse models globally while still supporting project-local installs when isolation is preferred.

## What Changes

- Add a download-scope prompt during model download that lets users choose `global` or `local` storage.
- Add global model path support in `.flashgrep` configuration and use it as the default lookup path when global scope is selected.
- Keep local project download behavior available for teams that want repository-scoped model storage.
- Update model resolution logic to check configured paths consistently and avoid duplicate downloads when a matching global model already exists.

## Capabilities

### New Capabilities
- `model-storage-scope`: Allow users to select global or local model installation at download time and resolve model paths from configuration.

### Modified Capabilities
- None.

## Impact

- Affected CLI download flow and interactive prompt behavior.
- Affected configuration parsing/validation for `.flashgrep` model path settings.
- Affected model path resolution and cache/reuse logic.
- Potential docs updates for model setup and configuration examples.
