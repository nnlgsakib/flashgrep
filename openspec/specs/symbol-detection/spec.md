## ADDED Requirements

### Requirement: Function detection
The symbol detector SHALL identify function-like definitions using regex patterns.

#### Scenario: Detect function definitions
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `function name`, `def name`, `fn name`, `func name`

#### Scenario: Extract function name
- **WHEN** a function pattern is matched
- **THEN** it SHALL extract and store the function identifier

### Requirement: Class detection
The symbol detector SHALL identify class-like blocks using regex patterns.

#### Scenario: Detect class definitions
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `class Name`, `struct Name`, `interface Name`, `type Name`

#### Scenario: Extract class name
- **WHEN** a class pattern is matched
- **THEN** it SHALL extract and store the class identifier

### Requirement: Import statement detection
The symbol detector SHALL identify import/include statements.

#### Scenario: Detect imports
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `import`, `require`, `include`, `use`, `from ... import`

#### Scenario: Store import metadata
- **WHEN** an import is detected
- **THEN** it SHALL store the imported module/path

### Requirement: Export statement detection
The symbol detector SHALL identify export statements.

#### Scenario: Detect exports
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `export`, `module.exports`, `pub fn`, `public`

### Requirement: Route definition detection
The symbol detector SHALL identify HTTP route definitions.

#### Scenario: Detect routes
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: route decorators, `.get(`, `.post(`, router definitions

### Requirement: SQL query detection
The symbol detector SHALL identify SQL query patterns.

#### Scenario: Detect SQL queries
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `SELECT`, `INSERT`, `UPDATE`, `DELETE` statements

### Requirement: Public/private marker detection
The symbol detector SHALL identify visibility modifiers.

#### Scenario: Detect visibility
- **WHEN** scanning source code
- **THEN** it SHALL detect patterns matching: `public`, `private`, `protected`, `pub`, `internal`

### Requirement: Language agnostic approach
The symbol detector SHALL work across all supported languages without language-specific parsers.

#### Scenario: Generic pattern matching
- **WHEN** processing any supported file type
- **THEN** it SHALL apply the same regex patterns regardless of language

#### Scenario: No parser dependencies
- **WHEN** indexing files
- **THEN** it SHALL NOT require language-specific parser libraries
