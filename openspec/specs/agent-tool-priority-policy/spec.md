## ADDED Requirements

### Requirement: Deterministic Flashgrep-first tool routing policy
The system MUST define a machine-readable tool routing policy that prioritizes Flashgrep-native tools for search, read, write, and symbol workflows before any native or generic alternatives.

#### Scenario: Policy declares prioritized tool families
- **WHEN** a client requests bootstrap policy metadata
- **THEN** the policy MUST include explicit preferred tool groups for query, glob/files, symbol, read_code, and write_code

#### Scenario: Native fallback is gate-controlled
- **WHEN** an agent attempts to use non-Flashgrep native tools
- **THEN** policy metadata MUST define allowed fallback gates and typed reasons for that decision

### Requirement: Policy includes compliance and observability metadata
The system MUST expose policy metadata fields that allow clients to verify enforcement mode and diagnose routing deviations.

#### Scenario: Enforcement mode is explicit
- **WHEN** policy metadata is returned
- **THEN** it MUST include an explicit policy strength or enforcement mode field

#### Scenario: Compliance state is inspectable
- **WHEN** an agent session consumes bootstrap policy metadata
- **THEN** response metadata MUST include fields that allow clients to report whether policy was injected and recognized
