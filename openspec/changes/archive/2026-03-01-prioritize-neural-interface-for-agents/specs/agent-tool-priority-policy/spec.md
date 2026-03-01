## MODIFIED Requirements

### Requirement: Deterministic Flashgrep-first tool routing policy
The system MUST define a machine-readable tool routing policy that prioritizes Flashgrep-native tools for search, read, write, and symbol workflows before any native or generic alternatives, and this policy MUST be delivered during initialization from a deterministic embedded source. For agent search workflows, this policy MUST explicitly prioritize neural retrieval first and programmatic retrieval second with typed fallback gates.

#### Scenario: Policy declares prioritized tool families
- **WHEN** a client requests bootstrap policy metadata
- **THEN** the policy MUST include explicit preferred tool groups for query, glob/files, symbol, read_code, and write_code

#### Scenario: Search policy declares neural-first retrieval order
- **WHEN** policy metadata includes search routing details
- **THEN** it MUST declare semantic/neural retrieval as primary for discovery intents
- **AND** it MUST declare programmatic/lexical retrieval as second priority

#### Scenario: Native fallback is gate-controlled
- **WHEN** an agent attempts to use non-Flashgrep native tools
- **THEN** policy metadata MUST define allowed fallback gates and typed reasons for that decision

#### Scenario: Native fallback requires explicit gating reason
- **WHEN** fallback metadata indicates a non-Flashgrep path was used
- **THEN** the metadata MUST include a specific gate identifier rather than a generic advisory
