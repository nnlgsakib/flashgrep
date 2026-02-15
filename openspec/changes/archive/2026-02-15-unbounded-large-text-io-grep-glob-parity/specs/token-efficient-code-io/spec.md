## MODIFIED Requirements

### Requirement: Budget-constrained code reads
The system MUST provide code read operations that enforce caller-provided `max_tokens`, `max_bytes`, and `max_lines` limits with deterministic truncation behavior and MUST support continuation loops to complete arbitrarily large logical reads.

#### Scenario: Read respects explicit limits
- **WHEN** a caller requests a file read with one or more explicit budget limits
- **THEN** the system returns code content that does not exceed enforced limits and reports applied limits in the response metadata

#### Scenario: Read indicates continuation when truncated
- **WHEN** a file read is truncated due to any enforced limit
- **THEN** the system returns continuation information that allows the caller to request the next segment without repeating already returned content

#### Scenario: Full logical read completed via continuation
- **WHEN** requested content is larger than one bounded response
- **THEN** the system MUST allow repeated continuation requests until the full requested scope is retrieved exactly

### Requirement: Minimal-diff writes with precondition safety
The system MUST provide write operations that apply replacements only within explicit line ranges, validate optional preconditions before mutating files, and support chunked replacement workflows for very large content.

#### Scenario: Write succeeds with matching preconditions
- **WHEN** a caller submits a line-range replacement and preconditions match current file state
- **THEN** the system applies the replacement atomically and returns success metadata including affected lines

#### Scenario: Write rejected on precondition mismatch
- **WHEN** a caller submits a line-range replacement and preconditions do not match current file state
- **THEN** the system rejects the write and returns structured conflict details sufficient for caller retry/rebase

#### Scenario: Oversized replacement can proceed as chunked workflow
- **WHEN** replacement content exceeds single-request safety size
- **THEN** the system MUST provide continuation-compatible write semantics so callers can complete the full logical replacement in deterministic chunks
