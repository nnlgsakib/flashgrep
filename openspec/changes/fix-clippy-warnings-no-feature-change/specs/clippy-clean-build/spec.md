## ADDED Requirements

### Requirement: Strict clippy compliance for full workspace targets
The project SHALL pass `cargo clippy --all-targets --all-features -- -D warnings` on supported development and CI environments.

#### Scenario: Clippy pass succeeds for all targets
- **WHEN** a maintainer runs `cargo clippy --all-targets --all-features -- -D warnings`
- **THEN** clippy SHALL complete without any warnings or errors

### Requirement: Lint compliance without feature behavior changes
Refactors introduced to satisfy clippy SHALL preserve existing user-visible behavior and public tool semantics.

#### Scenario: Feature parity after lint refactors
- **WHEN** lint-compliance changes are applied and verification commands are run
- **THEN** existing command behavior, protocol semantics, and test expectations SHALL remain unchanged
