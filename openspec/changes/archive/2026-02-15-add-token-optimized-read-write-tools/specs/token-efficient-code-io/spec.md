## ADDED Requirements

### Requirement: Budget-constrained code reads
The system MUST provide code read operations that enforce caller-provided `max_tokens`, `max_bytes`, and `max_lines` limits with deterministic truncation behavior.

#### Scenario: Read respects explicit limits
- **WHEN** a caller requests a file read with one or more explicit budget limits
- **THEN** the system returns code content that does not exceed enforced limits and reports applied limits in the response metadata

#### Scenario: Read indicates continuation when truncated
- **WHEN** a file read is truncated due to any enforced limit
- **THEN** the system returns continuation information that allows the caller to request the next segment without repeating already returned content

### Requirement: Targeted retrieval modes for minimal overfetch
The system MUST support both line-range reads and symbol-oriented reads so callers can retrieve only the required code scope.

#### Scenario: Caller requests specific line range
- **WHEN** a caller provides `file_path` with explicit start/end line bounds
- **THEN** the system returns only the requested slice subject to budget limits

#### Scenario: Caller requests symbol-scoped context
- **WHEN** a caller provides `symbol_name` with optional surrounding context lines
- **THEN** the system resolves the symbol definition and returns the scoped region subject to budget limits

### Requirement: Minimal-diff writes with precondition safety
The system MUST provide write operations that apply replacements only within explicit line ranges and validate optional preconditions before mutating files.

#### Scenario: Write succeeds with matching preconditions
- **WHEN** a caller submits a line-range replacement and preconditions match current file state
- **THEN** the system applies the replacement atomically and returns success metadata including affected lines

#### Scenario: Write rejected on precondition mismatch
- **WHEN** a caller submits a line-range replacement and preconditions do not match current file state
- **THEN** the system rejects the write and returns structured conflict details sufficient for caller retry/rebase

### Requirement: Token-efficient response metadata profiles
The system MUST support metadata verbosity levels that allow callers to reduce token overhead while preserving essential correctness signals.

#### Scenario: Minimal metadata profile selected
- **WHEN** a caller requests `metadata_level=minimal`
- **THEN** the system omits non-essential metadata fields and returns only required identifiers, limits, and continuation/conflict signals

#### Scenario: Standard metadata profile selected
- **WHEN** a caller requests `metadata_level=standard` or omits metadata level
- **THEN** the system returns the default diagnostic metadata defined for normal debugging and observability
