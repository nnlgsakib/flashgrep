## ADDED Requirements

### Requirement: Provider and model configuration for neural navigation
The system SHALL support provider configuration for neural navigation including provider id, base URL, model id, and API key source.

#### Scenario: Default OpenRouter profile is available
- **WHEN** neural navigation configuration is initialized
- **THEN** default values SHALL be set to provider `openrouter`, base URL `https://openrouter.ai/api/v1/chat/completions`, and model `arcee-ai/trinity-large-preview:free`

#### Scenario: User selects custom provider/model
- **WHEN** a user configures provider and model settings
- **THEN** the system SHALL validate required fields and persist deterministic configuration

### Requirement: API key resolution and safety
The system SHALL resolve provider credentials from explicit config or environment variables and SHALL fail with actionable diagnostics when credentials are missing.

#### Scenario: API key resolved from environment variable
- **WHEN** configuration references an API key environment variable
- **THEN** the system SHALL read and use that environment variable at request time

#### Scenario: Missing API key
- **WHEN** neural navigation requires a provider call and no valid API key is resolved
- **THEN** the system SHALL return a deterministic error with setup guidance and SHALL not execute an uncredentialed request
