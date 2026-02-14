## ADDED Requirements

### Requirement: Search Tool
The system SHALL provide an MCP tool for performing basic grep searches.

#### Scenario: Search for pattern in files
- **WHEN** user uses the `search` tool with a pattern and file paths
- **THEN** system searches for the pattern in specified files
- **AND** returns list of matching lines with file names and line numbers

#### Scenario: Search with case sensitivity
- **WHEN** user uses the `search` tool with case-sensitive option
- **THEN** search is performed with case sensitivity enabled
- **AND** only exact case matches are returned

#### Scenario: Search with regex
- **WHEN** user uses the `search` tool with regex option
- **THEN** search is performed using regular expressions
- **AND** regex patterns are properly matched

### Requirement: Search in Directory Tool
The system SHALL provide an MCP tool for searching within specific directories.

#### Scenario: Search directory recursively
- **WHEN** user uses the `search-in-directory` tool with a directory path
- **THEN** system searches recursively through the directory
- **AND** returns all matches found in files within the directory

#### Scenario: Search directory with file filters
- **WHEN** user uses the `search-in-directory` tool with file extension filters
- **THEN** system only searches files matching the specified extensions
- **AND** returns matches from filtered files

### Requirement: Search with Context Tool
The system SHALL provide an MCP tool for searching with context lines.

#### Scenario: Search with context
- **WHEN** user uses the `search-with-context` tool with context lines specified
- **THEN** system returns matching lines with specified number of context lines before and after
- **AND** context lines are properly formatted in the output

#### Scenario: Search with custom context
- **WHEN** user specifies custom before and after context lines
- **THEN** system returns matches with exact number of context lines requested
- **AND** context is clearly separated from matching lines

### Requirement: Search by Regex Tool
The system SHALL provide an MCP tool for regex-based searches.

#### Scenario: Regex search
- **WHEN** user uses the `search-by-regex` tool with a regex pattern
- **THEN** system searches for matches using regular expressions
- **AND** returns all lines matching the regex pattern

#### Scenario: Regex search with flags
- **WHEN** user uses the `search-by-regex` tool with regex flags (e.g., multiline)
- **THEN** regex engine uses the specified flags
- **AND** matches are found according to flag settings