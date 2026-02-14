## ADDED Requirements

### Requirement: Glob MCP contract supports advanced options
The MCP server MUST expose an expanded glob tool contract supporting advanced filtering, traversal, ordering, and limit controls.

#### Scenario: Advanced glob options are accepted
- **WHEN** a client calls glob with advanced options such as `extensions`, `exclude`, `max_depth`, `include_hidden`, `sort_by`, or `limit`
- **THEN** the server MUST validate and apply those options and return matching results

#### Scenario: Invalid option combinations return structured errors
- **WHEN** a client sends unsupported or incompatible glob option combinations
- **THEN** the server MUST return a structured parameter error without partial ambiguous behavior

### Requirement: Glob performance remains suitable for large repositories
The MCP server MUST apply traversal-time filtering and short-circuit strategies that reduce unnecessary scanning overhead.

#### Scenario: Early pruning with excludes and depth
- **WHEN** exclude filters or depth bounds are provided
- **THEN** the server MUST prune traversal early instead of scanning and post-filtering full trees
