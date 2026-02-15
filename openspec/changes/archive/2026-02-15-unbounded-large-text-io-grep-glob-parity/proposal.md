## Why

Users still see MCP connection failures and partial outcomes when operations involve very large reads or writes, which breaks trust in automation. Flashgrep needs to support arbitrarily large text operations with precise behavior while preserving grep/glob/read/write parity in real production workflows.

## What Changes

- Introduce unbounded large-text operation handling through chunked execution and continuation semantics so very large requests complete without transport crashes.
- Guarantee precise read/write behavior across large payloads, including deterministic ordering, exact line targeting, and no silent truncation.
- Expand grep/glob/read/write parity requirements so high-volume operations remain complete and accurate under heavy load.
- **BREAKING**: replace hard failure-only large-payload behavior with resumable multi-part response/write contracts where needed for transport safety.
- Update CLI and MCP docs to describe large-operation workflows, continuation loops, and compatibility guarantees.

## Capabilities

### New Capabilities
- `unbounded-large-text-operations`: End-to-end support for arbitrarily large text reads/writes/search traversals via resumable chunked processing with precise outcomes.

### Modified Capabilities
- `mcp-server`: Strengthen transport and request lifecycle requirements so large operations do not terminate sessions.
- `token-efficient-code-io`: Extend read/write requirements to guarantee precise multi-part continuation behavior for very large content.
- `glob-query-enhancements`: Extend glob behavior for complete large-repo traversal without loss while retaining deterministic output.
- `cli-search-commands`: Ensure CLI grep/glob/read workflows can orchestrate large continuation-based operations predictably.

## Impact

- Affected code: MCP stdio/TCP handlers, code IO handlers, query/glob execution paths, and CLI command surfaces.
- Affected API contracts: `read_code`, `write_code`, `get_slice`, `query`, and `glob` payload semantics for large operations.
- Affected docs/tests: README, skills guidance, and new large-scale regression suites for accuracy and connection stability.
