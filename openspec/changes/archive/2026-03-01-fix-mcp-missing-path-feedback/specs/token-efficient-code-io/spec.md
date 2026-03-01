## MODIFIED Requirements

### Requirement: Budget-constrained code reads
The system MUST provide code read operations that enforce caller-provided `max_tokens`, `max_bytes`, and `max_lines` limits with deterministic truncation behavior and MUST support continuation loops to complete arbitrarily large logical reads. When a requested file path is missing, the read operation MUST return a typed machine-readable not-found payload rather than ambiguous generic errors.

#### Scenario: Read respects explicit limits
- **WHEN** a caller requests a file read with one or more explicit budget limits
- **THEN** the system returns code content that does not exceed enforced limits and reports applied limits in the response metadata

#### Scenario: Read indicates continuation when truncated
- **WHEN** a file read is truncated due to any enforced limit
- **THEN** the system returns continuation information that allows the caller to request the next segment without repeating already returned content

#### Scenario: Full logical read completed via continuation
- **WHEN** requested content is larger than one bounded response
- **THEN** the system MUST allow repeated continuation requests until the full requested scope is retrieved exactly

#### Scenario: Missing file read target returns typed not-found
- **WHEN** a caller requests `read_code` or `get_slice` for a non-existent file path
- **THEN** the system MUST return a deterministic not-found error payload including target path and reason code
