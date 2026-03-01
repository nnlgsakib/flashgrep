## ADDED Requirements

### Requirement: Neural-first routing contract for agent discovery
The system SHALL define an agent routing contract that prioritizes neural retrieval for discovery-style search intents and SHALL only use programmatic retrieval as a second-choice fallback.

#### Scenario: Discovery intent uses neural first
- **WHEN** an agent issues a discovery-style search intent (natural-language location or relevance query)
- **THEN** the routing policy SHALL select neural retrieval before programmatic retrieval

#### Scenario: Programmatic search is second-choice only
- **WHEN** neural-first policy is active
- **THEN** programmatic retrieval SHALL execute only after a typed fallback gate is satisfied

### Requirement: Typed fallback reason codes
The system SHALL define deterministic fallback reason codes for routing from neural-first to programmatic retrieval.

#### Scenario: Model unavailable fallback
- **WHEN** neural model/runtime is unavailable for a request
- **THEN** policy metadata SHALL emit a typed fallback reason indicating model unavailability

#### Scenario: Exact-match fallback
- **WHEN** a query requires strict literal or regex semantics
- **THEN** policy metadata SHALL emit a typed fallback reason indicating exact-match requirement

#### Scenario: Low-confidence fallback
- **WHEN** neural retrieval confidence does not satisfy policy thresholds
- **THEN** policy metadata SHALL emit a typed fallback reason indicating low-confidence routing
