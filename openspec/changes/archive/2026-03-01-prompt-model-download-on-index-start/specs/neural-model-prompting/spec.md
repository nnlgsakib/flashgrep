## ADDED Requirements

### Requirement: Startup model prompt for missing neural assets
The system SHALL check for `BAAI/bge-small-en-v1.5` model cache before startup flows that perform indexing and SHALL prompt the user to download when the cache is missing.

#### Scenario: Prompt shown when model is missing
- **WHEN** indexing startup flow begins and `.flashgrep/model-cache/BAAI__bge-small-en-v1.5` is missing
- **THEN** the system SHALL prompt the user with a yes/no download choice before continuing

#### Scenario: Accept download prompt
- **WHEN** the user answers `y`
- **THEN** the system SHALL download model assets to `.flashgrep/model-cache/`
- **AND** it SHALL continue the startup flow after successful download

#### Scenario: Decline download prompt
- **WHEN** the user answers `n`
- **THEN** the system SHALL continue startup flow without downloading model assets
- **AND** it SHALL preserve normal lexical indexing behavior

### Requirement: Deterministic non-interactive fallback
The system SHALL avoid blocking startup in non-interactive environments.

#### Scenario: Non-interactive startup
- **WHEN** startup flow runs without interactive stdin available
- **THEN** the system SHALL skip prompting and continue without model download
- **AND** it SHALL emit a clear informational message describing how to install the model later
