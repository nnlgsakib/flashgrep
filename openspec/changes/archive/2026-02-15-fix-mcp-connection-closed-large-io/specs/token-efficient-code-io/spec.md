## MODIFIED Requirements

### Requirement: Budget-constrained code reads
The system MUST provide code read operations that enforce caller-provided `max_tokens`, `max_bytes`, and `max_lines` limits with deterministic truncation behavior and MUST avoid generating responses that destabilize MCP transport sessions.

#### Scenario: Read respects explicit limits
- **WHEN** a caller requests a file read with one or more explicit budget limits
- **THEN** the system returns code content that does not exceed enforced limits and reports applied limits in the response metadata

#### Scenario: Read indicates continuation when truncated
- **WHEN** a file read is truncated due to any enforced limit
- **THEN** the system returns continuation information that allows the caller to request the next segment without repeating already returned content

#### Scenario: Read request beyond safety bounds returns structured limit error
- **WHEN** a caller omits limits or requests a range that exceeds server safety bounds for MCP response payloads
- **THEN** the system MUST return a structured size-limit error or bounded truncated result and MUST keep the MCP session open

### Requirement: Minimal-diff writes with precondition safety
The system MUST provide write operations that apply replacements only within explicit line ranges, validate optional preconditions before mutating files, and reject oversized replacements with machine-actionable errors.

#### Scenario: Write succeeds with matching preconditions
- **WHEN** a caller submits a line-range replacement and preconditions match current file state
- **THEN** the system applies the replacement atomically and returns success metadata including affected lines

#### Scenario: Write rejected on precondition mismatch
- **WHEN** a caller submits a line-range replacement and preconditions do not match current file state
- **THEN** the system rejects the write and returns structured conflict details sufficient for caller retry/rebase

#### Scenario: Oversized write replacement is rejected without transport failure
- **WHEN** a caller submits replacement content larger than configured write payload limits
- **THEN** the system MUST return a structured size-limit error including maximum allowed replacement size and MUST NOT terminate the MCP connection
