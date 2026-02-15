## ADDED Requirements

### Requirement: Search files by glob pattern
The system SHALL provide a command to search for files using glob patterns.

#### Scenario: Search with simple pattern
- **WHEN** user executes fs-search with pattern="*.txt" and path="/project"
- **THEN** system returns all .txt files in /project directory

#### Scenario: Search with recursive pattern
- **WHEN** user executes fs-search with pattern="**/*.js" and path="/src"
- **THEN** system returns all .js files in /src and all subdirectories

#### Scenario: Search with multiple patterns
- **WHEN** user executes fs-search with patterns=["*.ts", "*.tsx"] and path="/src"
- **THEN** system returns all files matching any pattern
- **AND** returns combined unique results

#### Scenario: Search with exclusion pattern
- **WHEN** user executes fs-search with pattern="**/*.js" and exclude=["node_modules/**", "*.test.js"]
- **THEN** system returns all .js files except those in node_modules or ending with .test.js

### Requirement: Search with content filtering
The system SHALL support filtering search results by file content.

#### Scenario: Search files containing text
- **WHEN** user executes fs-search with pattern="*.md" and contains="TODO"
- **THEN** system returns markdown files that contain the text "TODO"

#### Scenario: Search files matching regex
- **WHEN** user executes fs-search with pattern="*.log" and regex="ERROR\s*\d{4}"
- **THEN** system returns log files containing lines matching the regex pattern

### Requirement: Return structured search results
The system SHALL return detailed information about found files.

#### Scenario: Basic file information
- **WHEN** user executes fs-search with pattern="*.json"
- **THEN** system returns array of results
- **AND** each result includes: path, name, size, modifiedTime

#### Scenario: Include file content
- **WHEN** user executes fs-search with pattern="*.md" and includeContent=true
- **THEN** system returns results with file content included
- **AND** respects encoding parameter for text files

#### Scenario: Limit result count
- **WHEN** user executes fs-search with pattern="**/*" and limit=100
- **THEN** system returns at most 100 results
- **AND** stops searching after reaching limit

### Requirement: Sort search results
The system SHALL support sorting search results by various criteria.

#### Scenario: Sort by name
- **WHEN** user executes fs-search with pattern="*.txt" and sortBy="name"
- **THEN** system returns results sorted alphabetically by filename

#### Scenario: Sort by modification time
- **WHEN** user executes fs-search with pattern="*.log" and sortBy="modifiedTime" and sortOrder="desc"
- **THEN** system returns results sorted by modification time, newest first

#### Scenario: Sort by file size
- **WHEN** user executes fs-search with pattern="*.bin" and sortBy="size" and sortOrder="desc"
- **THEN** system returns results sorted by file size, largest first
