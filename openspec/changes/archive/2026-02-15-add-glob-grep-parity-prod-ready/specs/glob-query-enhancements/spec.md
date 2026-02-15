## MODIFIED Requirements

### Requirement: Comprehensive glob filtering options
The system MUST provide glob discovery with composable filtering options including include/exclude path filters, extension filters, hidden-file behavior, symlink-follow policy, and recursion/depth controls.

#### Scenario: Filter by extension and exclude paths in one call
- **WHEN** a caller provides include pattern(s), extension filter(s), and exclude pattern(s)
- **THEN** the glob result MUST include only files matching include and extension constraints while omitting excluded paths

#### Scenario: Bound traversal depth
- **WHEN** a caller provides `max_depth`
- **THEN** the glob traversal MUST NOT return matches deeper than the requested depth

#### Scenario: Hidden and symlink behavior is explicit
- **WHEN** a caller provides `include_hidden` and `follow_symlinks` options
- **THEN** the traversal MUST apply those options consistently and document defaults when options are omitted

### Requirement: Deterministic ordering and limiting
The system MUST support deterministic sorting and bounded limiting (and offset/pagination where supported) for predictable high-volume discovery.

#### Scenario: Sort and limit results
- **WHEN** a caller provides `sort_by`, `sort_order`, and `limit`
- **THEN** the response MUST return at most `limit` items in deterministic order according to the requested sort settings

#### Scenario: Stable pagination windows
- **WHEN** callers retrieve adjacent windows using deterministic sort settings and offsets
- **THEN** results MUST remain stable for unchanged indexes across repeated calls
