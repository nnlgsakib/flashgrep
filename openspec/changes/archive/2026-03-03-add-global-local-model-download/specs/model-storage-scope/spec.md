## ADDED Requirements

### Requirement: Model download scope selection
The system SHALL prompt interactive users to choose model storage scope when a model download is required and the user has accepted downloading.

#### Scenario: User selects global scope
- **WHEN** the model is missing, the user accepts download, and chooses `global`
- **THEN** the system resolves the configured global model cache path and downloads the model into that location

#### Scenario: User selects local scope
- **WHEN** the model is missing, the user accepts download, and chooses `local`
- **THEN** the system downloads the model into the repository-local `.flashgrep/model-cache` location

### Requirement: Config-driven global model cache path
The system SHALL support a `.flashgrep` configuration field that defines a global model cache path used when scope is `global`.

#### Scenario: Global path is configured
- **WHEN** scope is `global` and the configuration contains a valid global model cache path
- **THEN** cache checks, manifest reads, and download targets use the configured global path

#### Scenario: Global path is missing
- **WHEN** scope is `global` and no global model cache path is configured
- **THEN** the system reports a clear configuration error with guidance to set the path or choose local scope

### Requirement: Reuse global model across projects
The system SHALL avoid re-downloading a model when the selected cache location already contains a valid manifest for the required model ID.

#### Scenario: Model already exists in global cache
- **WHEN** scope is `global` and the configured global cache already has a valid manifest for the required model ID
- **THEN** the system treats the model as cached and skips download

### Requirement: Non-interactive deterministic behavior
The system SHALL not prompt for scope in non-interactive mode and MUST use deterministic cache-scope resolution.

#### Scenario: Non-interactive run without explicit scope
- **WHEN** stdin/stdout are non-interactive and no explicit scope override is provided
- **THEN** the system uses configured default scope resolution and continues without prompting
