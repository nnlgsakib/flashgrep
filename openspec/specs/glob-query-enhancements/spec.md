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

### Requirement: Deterministic ordering and limiting
The system MUST support deterministic sorting and result limiting for predictable high-volume discovery and MUST preserve deterministic ordering across continuation windows and platforms.

#### Scenario: Sort and limit results
- **WHEN** a caller provides `sort_by`, `sort_order`, and `limit`
- **THEN** the response MUST return at most `limit` items in deterministic order according to the requested sort settings

#### Scenario: Stable continuation ordering
- **WHEN** a caller paginates through continuation windows with stable filters and sort configuration
- **THEN** the system MUST preserve deterministic no-gap/no-duplicate ordering for the full result set

#### Scenario: Cross-platform deterministic ordering
- **WHEN** equivalent glob inputs are executed on supported platforms against equivalent trees
- **THEN** ordering and limit behavior MUST remain deterministic under documented normalization rules

## ADDED Requirements

### Requirement: Shell-glob parity for common production patterns
Glob matching MUST support common shell-style production patterns required for replacement workflows, including recursive wildcards and character class matching under documented semantics.

#### Scenario: Recursive wildcard traversal
- **WHEN** a caller uses recursive wildcard patterns
- **THEN** matching MUST include all descendants that satisfy the pattern subject to configured depth and exclusion controls

#### Scenario: Character class pattern matching
- **WHEN** a caller uses character class syntax in a glob pattern
- **THEN** matching MUST evaluate the class according to documented glob rules and return deterministic results
