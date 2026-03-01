## MODIFIED Requirements

### Requirement: Comprehensive glob filtering options
The system MUST provide glob discovery with composable filtering options including include/exclude path filters, extension filters, hidden-file behavior, recursion/depth controls, and deterministic cross-platform path normalization, while supporting continuation over arbitrarily large result sets. If the caller-provided search root path does not exist, the tool MUST return typed not-found diagnostics.

#### Scenario: Filter by extension and exclude paths in one call
- **WHEN** a caller provides include pattern(s), extension filter(s), and exclude pattern(s)
- **THEN** the glob result MUST include only files matching include and extension constraints while omitting excluded paths

#### Scenario: Bound traversal depth
- **WHEN** a caller provides `max_depth`
- **THEN** the glob traversal MUST NOT return matches deeper than the requested depth

#### Scenario: Continuation window over very large repos
- **WHEN** total matches exceed one response window
- **THEN** the system MUST provide deterministic continuation windows until all matching paths are retrievable

#### Scenario: Hidden and dot-path matching policy
- **WHEN** a caller explicitly enables hidden-file matching
- **THEN** glob results MUST include hidden and dot-prefixed paths according to documented policy

#### Scenario: Missing glob root path returns typed not-found
- **WHEN** a caller provides a non-existent root `path` for glob discovery
- **THEN** the system MUST return a deterministic not-found payload identifying the missing directory path
