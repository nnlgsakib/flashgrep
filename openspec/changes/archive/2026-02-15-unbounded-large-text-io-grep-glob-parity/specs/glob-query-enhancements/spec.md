## MODIFIED Requirements

### Requirement: Comprehensive glob filtering options
The system MUST provide glob discovery with composable filtering options including include/exclude path filters, extension filters, hidden-file behavior, and recursion/depth controls, while supporting continuation over arbitrarily large result sets.

#### Scenario: Filter by extension and exclude paths in one call
- **WHEN** a caller provides include pattern(s), extension filter(s), and exclude pattern(s)
- **THEN** the glob result MUST include only files matching include and extension constraints while omitting excluded paths

#### Scenario: Bound traversal depth
- **WHEN** a caller provides `max_depth`
- **THEN** the glob traversal MUST NOT return matches deeper than the requested depth

#### Scenario: Continuation window over very large repos
- **WHEN** total matches exceed one response window
- **THEN** the system MUST provide deterministic continuation windows until all matching paths are retrievable

### Requirement: Deterministic ordering and limiting
The system MUST support deterministic sorting and result limiting for predictable high-volume discovery and must preserve deterministic ordering across continuation windows.

#### Scenario: Sort and limit results
- **WHEN** a caller provides `sort_by`, `sort_order`, and `limit`
- **THEN** the response MUST return at most `limit` items in deterministic order according to the requested sort settings

#### Scenario: Stable continuation ordering
- **WHEN** a caller paginates through continuation windows with stable filters and sort configuration
- **THEN** the system MUST preserve deterministic no-gap/no-duplicate ordering for the full result set
