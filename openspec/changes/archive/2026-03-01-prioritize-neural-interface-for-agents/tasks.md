## 1. Policy Schema and Bootstrap Metadata

- [x] 1.1 Add neural-first retrieval ordering fields to bootstrap policy metadata for agent search routing.
- [x] 1.2 Add typed fallback gate/reason codes for programmatic second-choice routing (model unavailable, low confidence, exact-match requirement, parse constraints).
- [x] 1.3 Ensure policy metadata preserves deterministic compliance/observability fields across bootstrap handlers.

## 2. Skill Guidance and Embedded Payload Updates

- [x] 2.1 Update embedded bootstrap skill payload to instruct neural-first search for discovery intents.
- [x] 2.2 Update agent skill bootstrap responses to include neural-first guidance and programmatic fallback constraints.
- [x] 2.3 Keep idempotent bootstrap semantics unchanged while returning equivalent neural-first policy guidance on repeated init.

## 3. Agent Documentation and Fallback Semantics

- [x] 3.1 Update AI agent documentation to define neural-first usage as default search behavior.
- [x] 3.2 Document deterministic fallback decision criteria and reason-code expectations.
- [x] 3.3 Add troubleshooting guidance for policy drift and routing compliance checks.

## 4. Validation and Regression Coverage

- [x] 4.1 Add/extend tests that verify bootstrap policy metadata exposes neural-first ordering.
- [x] 4.2 Add/extend tests that verify fallback gates/reasons are typed and deterministic.
- [x] 4.3 Add/extend tests for consistency across stdio and TCP bootstrap surfaces.
