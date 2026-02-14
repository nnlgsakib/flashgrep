## Purpose

Define requirements for AI agent documentation that enables effective use of flashgrep MCP tools.

## ADDED Requirements

### Requirement: AI agent skill documentation structure
The system SHALL provide skill documentation in the OpenCode skill format.

#### Scenario: Skill file location
- **WHEN** an AI agent looks for flashgrep MCP documentation
- **THEN** it SHALL find the skill at `.opencode/skills/flashgrep-mcp/SKILL.md`

#### Scenario: Skill file format
- **WHEN** the skill file is read
- **THEN** it SHALL contain markdown formatted documentation
- **AND** it SHALL include sections for tools, patterns, and examples

### Requirement: MCP tool documentation
The system SHALL document each MCP tool with purpose, parameters, and examples.

#### Scenario: Query tool documentation
- **WHEN** AI agent needs to search code
- **THEN** the skill SHALL explain the `query` tool
- **AND** it SHALL provide example queries for different use cases

#### Scenario: Get slice tool documentation
- **WHEN** AI agent needs to read specific file lines
- **THEN** the skill SHALL explain the `get_slice` tool
- **AND** it SHALL show how to specify file path and line ranges

#### Scenario: Get symbol tool documentation
- **WHEN** AI agent needs to find symbol definitions
- **THEN** the skill SHALL explain the `get_symbol` tool
- **AND** it SHALL provide examples for functions, classes, and variables

#### Scenario: List files tool documentation
- **WHEN** AI agent needs to see indexed files
- **THEN** the skill SHALL explain the `list_files` tool
- **AND** it SHALL show example output

#### Scenario: Stats tool documentation
- **WHEN** AI agent needs index statistics
- **THEN** the skill SHALL explain the `stats` tool
- **AND** it SHALL describe what statistics are available

### Requirement: Search patterns and workflows
The system SHALL provide common search patterns for AI agents.

#### Scenario: Finding function definitions
- **WHEN** AI agent needs to find a function
- **THEN** the skill SHALL provide the pattern "fn function_name"
- **AND** it SHALL explain how to use context lines

#### Scenario: Finding class definitions
- **WHEN** AI agent needs to find a class or struct
- **THEN** the skill SHALL provide patterns like "class Name" or "struct Name"
- **AND** it SHALL show language-specific variations

#### Scenario: Searching imports
- **WHEN** AI agent needs to find where a module is imported
- **THEN** the skill SHALL provide patterns for import statements
- **AND** it SHALL explain how to search across file types

### Requirement: Error handling guidance
The system SHALL document common errors and recovery strategies.

#### Scenario: Connection errors
- **WHEN** AI agent encounters MCP connection errors
- **THEN** the skill SHALL explain common causes
- **AND** it SHALL provide troubleshooting steps

#### Scenario: Empty results
- **WHEN** AI agent gets empty search results
- **THEN** the skill SHALL suggest alternative search strategies
- **AND** it SHALL explain how to verify the index exists

### Requirement: Best practices
The system SHALL provide best practices for using flashgrep effectively.

#### Scenario: Query optimization
- **WHEN** AI agent searches for code
- **THEN** the skill SHALL recommend specific search terms
- **AND** it SHALL explain how to avoid overly broad queries

#### Scenario: Multi-step analysis
- **WHEN** AI agent needs to analyze complex code
- **THEN** the skill SHALL suggest breaking it into steps
- **AND** it SHALL provide example workflows

### Requirement: README MCP stdio setup documentation
The system SHALL document a complete MCP stdio setup flow for Flashgrep in `README.md`.

#### Scenario: Full MCP client config example
- **WHEN** a user needs to configure an MCP client for Flashgrep
- **THEN** the README SHALL include a full JSON configuration block with a `flashgrep` entry
- **AND** the config SHALL use command `flashgrep mcp-stdio`
- **AND** the config SHALL show `enabled: true`
- **AND** the config SHALL include `RUST_LOG: info` in environment settings

#### Scenario: Setup steps include validation
- **WHEN** a user follows README MCP setup instructions
- **THEN** the steps SHALL include indexing the repository before MCP usage
- **AND** the steps SHALL include launching stdio mode
- **AND** the steps SHALL include a simple validation outcome for successful connection

### Requirement: README skill discovery guidance
The system SHALL document agent-agnostic skill file locations and usage guidance.

#### Scenario: Generic skill path documented
- **WHEN** a user looks for a skill that works with any coding agent
- **THEN** the README SHALL reference `skills/SKILL.md`
- **AND** the README SHALL describe this as the primary, agent-agnostic skill path

#### Scenario: Optional OpenCode-managed path documented
- **WHEN** a user uses OpenCode-managed skills
- **THEN** the README SHALL reference `.opencode/skills/flashgrep-mcp/SKILL.md`
- **AND** the README SHALL describe this path as optional and OpenCode-specific

#### Scenario: Cross-linking from MCP API section
- **WHEN** a user reads the MCP API methods section
- **THEN** the README SHALL link to the setup and skill discovery section to reduce duplication

### Requirement: Native bootstrap usage documentation
The documentation set MUST describe how agents invoke native Flashgrep bootstrap to receive skill injection without manual external skill loading.

#### Scenario: Bootstrap trigger guidance is documented
- **WHEN** users read Flashgrep agent documentation
- **THEN** documentation MUST include bootstrap trigger examples for `flashgrep-init` and `fgrep-boot`

#### Scenario: Bootstrap response behavior is documented
- **WHEN** users read bootstrap documentation
- **THEN** documentation MUST describe first-call injection and repeated-call idempotent behavior

### Requirement: Flashgrep-first tool selection guidance
The documentation set MUST explicitly guide agents to prioritize Flashgrep-native tools over generic grep/glob workflows for matching tasks.

#### Scenario: Guidance includes preferred tool order
- **WHEN** users read the skill or bootstrap guidance
- **THEN** it MUST recommend using `query`, `files`, `symbol`, `read_code`, and `write_code` as primary operations

#### Scenario: Efficient read/write guidance is present
- **WHEN** users review read/write recommendations
- **THEN** documentation MUST describe budgeted `read_code` usage and targeted `write_code` usage for token-efficient agent operation

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
