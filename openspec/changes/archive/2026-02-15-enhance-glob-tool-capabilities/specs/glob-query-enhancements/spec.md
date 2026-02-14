## ADDED Requirements

### Requirement: Comprehensive glob filtering options
The system MUST provide glob discovery with composable filtering options including include/exclude path filters, extension filters, hidden-file behavior, and recursion/depth controls.

#### Scenario: Filter by extension and exclude paths in one call
- **WHEN** a caller provides include pattern(s), extension filter(s), and exclude pattern(s)
- **THEN** the glob result MUST include only files matching include and extension constraints while omitting excluded paths

#### Scenario: Bound traversal depth
- **WHEN** a caller provides `max_depth`
- **THEN** the glob traversal MUST NOT return matches deeper than the requested depth

### Requirement: Deterministic ordering and limiting
The system MUST support deterministic sorting and result limiting for predictable high-volume discovery.

#### Scenario: Sort and limit results
- **WHEN** a caller provides `sort_by`, `sort_order`, and `limit`
- **THEN** the response MUST return at most `limit` items in deterministic order according to the requested sort settings

### Requirement: Backward-compatible defaults
The system MUST preserve existing glob behavior when advanced options are not supplied.

#### Scenario: Legacy-style glob call
- **WHEN** a caller invokes glob using only the existing minimal parameters
- **THEN** results MUST remain behaviorally compatible with prior glob semantics
