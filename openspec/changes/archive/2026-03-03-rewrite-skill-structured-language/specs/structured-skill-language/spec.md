## ADDED Requirements

### Requirement: Structured skill syntax for token-efficient guidance
`skills/SKILL.md` SHALL use a compact structured language with deterministic keyword blocks so that guidance remains machine-parseable and token-efficient while preserving current policy semantics.

#### Scenario: Structured block format is used
- **WHEN** `skills/SKILL.md` is authored for bootstrap guidance
- **THEN** the document SHALL use compact directive-style blocks (for example `TASK`, `MODE`, `TOOLS`, `FALLBACK`, `RULE`, `EXAMPLE`) instead of long prose paragraphs

#### Scenario: Edit actions are represented compactly
- **WHEN** the skill provides editing workflow examples
- **THEN** examples SHALL support concise structured edit patterns using fields like `FILE`, `FIND`, and `REPLACE`

### Requirement: Behavior parity under compact format
The structured format MUST preserve all existing operational behavior and policy constraints from the prior skill guidance.

#### Scenario: Tool-order semantics remain unchanged
- **WHEN** agents use structured guidance
- **THEN** the document MUST preserve Flashgrep-first routing and neural-first discovery ordering with deterministic lexical fallback

#### Scenario: Safety and compliance semantics remain unchanged
- **WHEN** agents evaluate fallback and native-tool usage
- **THEN** the document MUST preserve required fallback reason codes, compliance recovery steps, and native-tool ban constraints
