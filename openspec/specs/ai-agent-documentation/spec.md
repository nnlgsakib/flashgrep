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
