# model-storage-scope Specification

## Purpose
TBD - created by archiving change add-global-local-model-download. Update Purpose after archive.
## Requirements
### Requirement: Model download scope selection
The system SHALL prompt interactive users to choose model storage scope when a model download is required and the user has accepted downloading.

#### Scenario: User selects global scope
- **WHEN** the model is missing, the user accepts download, and chooses `global`
- **THEN** the system resolves the configured global model cache path and downloads the model into that location

#### Scenario: User selects local scope
- **WHEN** the model is missing, the user accepts download, and chooses `local`
- **THEN** the system downloads the model into the repository-local `.flashgrep/model-cache` location

### Requirement: Config-driven global model cache path
The system SHALL support `.flashgrep` configuration fields for neural mode including global model cache path, provider id, base URL, model id, and API key source metadata.

#### Scenario: Global path and provider are configured
- **WHEN** scope is `global` and configuration contains valid cache path and provider settings
- **THEN** cache checks, manifest reads, and provider request configuration SHALL use configured values

#### Scenario: Required provider config is missing
- **WHEN** neural mode is enabled and required provider configuration is absent
- **THEN** the system SHALL report a clear configuration error with guidance to set provider/model/key fields or disable neural mode

### Requirement: Reuse global model across projects
The system SHALL avoid re-downloading a model when the selected cache location already contains a valid manifest for the required model ID.

#### Scenario: Model already exists in global cache
- **WHEN** scope is `global` and the configured global cache already has a valid manifest for the required model ID
- **THEN** the system treats the model as cached and skips download

### Requirement: Non-interactive deterministic behavior
The system SHALL not prompt for scope or neural enablement in non-interactive mode and MUST use deterministic config resolution.

#### Scenario: Non-interactive run without explicit overrides
- **WHEN** stdin/stdout are non-interactive and no explicit scope or neural overrides are provided
- **THEN** the system SHALL use configured defaults and continue without prompting
