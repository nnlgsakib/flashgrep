## ADDED Requirements

### Requirement: Generate comprehensive skill definition
The system SHALL generate a skill markdown document describing all flashgrep tools.

#### Scenario: Create skill with tool descriptions
- **WHEN** skill definition is generated
- **THEN** it SHALL include frontmatter with name, description, and license
- **AND** it SHALL describe when to use flashgrep tools
- **AND** it SHALL list all available tools (query, files, symbol, read_code, write_code)

#### Scenario: Include usage examples
- **WHEN** skill definition is generated
- **THEN** it SHALL include "Do" examples showing correct usage
- **AND** it SHALL include "Don't" examples showing incorrect usage
- **AND** each example SHALL be concrete and actionable

### Requirement: Tool-specific guidance in skill
The skill SHALL provide specific guidance for each flashgrep tool.

#### Scenario: Query tool guidance
- **WHEN** skill describes the query tool
- **THEN** it SHALL explain when to use semantic search
- **AND** it SHALL provide example queries for different use cases
- **AND** it SHALL explain the difference between smart and literal modes

#### Scenario: Files tool guidance
- **WHEN** skill describes the files tool
- **THEN** it SHALL explain glob pattern syntax
- **AND** it SHALL provide examples of file discovery patterns
- **AND** it SHALL explain when to use files vs query tool

#### Scenario: Symbol tool guidance
- **WHEN** skill describes the symbol tool
- **THEN** it SHALL explain symbol resolution use cases
- **AND** it SHALL provide examples of symbol lookups
- **AND** it SHALL explain the context lines parameter

#### Scenario: Read_code tool guidance
- **WHEN** skill describes the read_code tool
- **THEN** it SHALL explain token-efficient code reading
- **AND** it SHALL provide examples of reading by symbol or line range
- **AND** it SHALL explain continuation for large files

#### Scenario: Write_code tool guidance
- **WHEN** skill describes the write_code tool
- **THEN** it SHALL explain minimal-diff writing
- **AND** it SHALL provide examples of safe code modifications
- **AND** it SHALL explain precondition checking

### Requirement: Include keywords for context matching
The skill SHALL include keywords to help AI agents match context.

#### Scenario: Keywords section present
- **WHEN** skill definition is generated
- **THEN** it SHALL include a Keywords section
- **AND** it SHALL list relevant search terms (search, grep, find, locate, etc.)
- **AND** it SHALL include file operation keywords (read, write, edit, modify)

### Requirement: Skill priority and fallback guidance
The skill SHALL include guidance on tool priority and fallbacks.

#### Scenario: Priority guidance included
- **WHEN** skill describes tool selection
- **THEN** it SHALL state that flashgrep tools are preferred over generic tools
- **AND** it SHALL explain when native agent tools are acceptable fallbacks

#### Scenario: Anti-patterns documented
- **WHEN** skill provides guidance
- **THEN** it SHALL document common anti-patterns (e.g., using grep instead of query)
- **AND** it SHALL explain why flashgrep tools are superior for each use case
