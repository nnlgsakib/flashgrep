## ADDED Requirements

### Requirement: Advanced glob usage documentation
The documentation set MUST describe advanced glob parameters and practical combinations for efficient file discovery.

#### Scenario: Filter-driven examples are documented
- **WHEN** users consult Flashgrep docs/skills for glob usage
- **THEN** documentation MUST include examples for extension filtering, include/exclude patterns, and depth-bounded traversal

#### Scenario: Ordering and limit guidance is documented
- **WHEN** users need predictable bounded result sets
- **THEN** documentation MUST explain sorting and limit options for deterministic outputs

### Requirement: Efficiency guidance for glob workflows
The documentation set MUST guide users and agents on when to use advanced glob filters to reduce follow-up calls and scanning overhead.

#### Scenario: Guidance includes one-pass discovery strategy
- **WHEN** users read glob best practices
- **THEN** documentation MUST recommend combining filters in one call where possible to improve speed and efficiency
