## ADDED Requirements

### Requirement: Query MCP contract supports grep-parity options
The MCP server MUST expose query parameters for grep-parity behavior, including regex/literal mode, case handling, path scoping, and context controls, with structured validation errors.

#### Scenario: Query accepts parity parameters
- **WHEN** a client calls query with grep-parity options
- **THEN** the server MUST validate and apply those options and return matching structured results

#### Scenario: Query rejects invalid combinations deterministically
- **WHEN** a client sends unsupported or conflicting query option combinations
- **THEN** the server MUST return a structured parameter error with no ambiguous partial behavior

### Requirement: Glob MCP contract is replacement-grade for discovery workflows
The MCP server MUST expose glob/file-discovery parameters that cover production glob workflows, including include/exclude sets, extension filtering, hidden/symlink policy, depth bounds, deterministic sort, and bounded windows.

#### Scenario: Discovery parameters support one-pass filtering
- **WHEN** a client sends include, exclude, extension, and depth parameters together
- **THEN** the server MUST apply all filters in one request and return only compliant paths

#### Scenario: Result ordering and bounds are script-safe
- **WHEN** a client includes sorting and limit/window controls
- **THEN** the server MUST return deterministic bounded results suitable for automation
