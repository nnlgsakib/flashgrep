## MODIFIED Requirements

### Requirement: Deterministic Flashgrep-first tool routing policy
The system MUST define a machine-readable tool routing policy that prioritizes Flashgrep-native tools for search, read, write, and symbol workflows before any native or generic alternatives, and this policy MUST be delivered during initialization from a deterministic embedded source.

#### Scenario: Policy declares prioritized tool families
- **WHEN** a client requests bootstrap policy metadata
- **THEN** the policy MUST include explicit preferred tool groups for query, glob/files, symbol, read_code, and write_code

#### Scenario: Native fallback is gate-controlled
- **WHEN** an agent attempts to use non-Flashgrep native tools
- **THEN** policy metadata MUST define allowed fallback gates and typed reasons for that decision

#### Scenario: Native fallback requires explicit gating reason
- **WHEN** fallback metadata indicates a non-Flashgrep path was used
- **THEN** the metadata MUST include a specific gate identifier rather than a generic advisory

### Requirement: Policy includes compliance and observability metadata
The system MUST expose policy metadata fields that allow clients to verify enforcement mode, payload source, and diagnose routing deviations.

#### Scenario: Enforcement mode is explicit
- **WHEN** policy metadata is returned
- **THEN** it MUST include an explicit policy strength or enforcement mode field

#### Scenario: Compliance state is inspectable
- **WHEN** an agent session consumes bootstrap policy metadata
- **THEN** response metadata MUST include fields that allow clients to report whether policy was injected and recognized

#### Scenario: Payload source and bootstrap path are inspectable
- **WHEN** bootstrap metadata is returned
- **THEN** metadata MUST include skill payload source and bootstrap state fields sufficient to troubleshoot policy drift
