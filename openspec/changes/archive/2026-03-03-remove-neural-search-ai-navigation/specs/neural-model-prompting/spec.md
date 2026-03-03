## REMOVED Requirements

### Requirement: Startup model prompt for missing neural assets
**Reason**: Neural model assets are no longer part of product behavior.
**Migration**: Remove model-download prompt expectations from startup flows and related tests.

### Requirement: Deterministic non-interactive fallback
**Reason**: Non-interactive neural prompt fallback is obsolete after neural prompt removal.
**Migration**: Startup should proceed with lexical behavior only, without neural-install guidance messaging.
