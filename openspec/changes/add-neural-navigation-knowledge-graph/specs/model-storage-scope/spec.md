## MODIFIED Requirements

### Requirement: Config-driven global model cache path
The system SHALL support `.flashgrep` configuration fields for neural mode including global model cache path, provider id, base URL, model id, and API key source metadata.

#### Scenario: Global path and provider are configured
- **WHEN** scope is `global` and configuration contains valid cache path and provider settings
- **THEN** cache checks, manifest reads, and provider request configuration SHALL use configured values

#### Scenario: Required provider config is missing
- **WHEN** neural mode is enabled and required provider configuration is absent
- **THEN** the system SHALL report a clear configuration error with guidance to set provider/model/key fields or disable neural mode

### Requirement: Non-interactive deterministic behavior
The system SHALL not prompt for scope or neural enablement in non-interactive mode and MUST use deterministic config resolution.

#### Scenario: Non-interactive run without explicit overrides
- **WHEN** stdin/stdout are non-interactive and no explicit scope or neural overrides are provided
- **THEN** the system SHALL use configured defaults and continue without prompting
