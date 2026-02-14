## Why

The project currently has strong search capabilities but lacks a purpose-built read/write path optimized for token-constrained agents. This causes unnecessary token spend and slower codebase operations when agents need precise slices or minimal-diff edits.

## What Changes

- Add a new token-efficient code I/O capability with deterministic, bounded reads and minimal-diff writes.
- Introduce read operations that support strict byte/line/token budgets, continuation cursors, and optional symbol-scoped retrieval.
- Introduce write operations that apply targeted line-range replacements with precondition checks to prevent stale edits.
- Add concise metadata modes to reduce response overhead while preserving enough context for safe agent use.
- Define validation and conflict-reporting behavior so agents can recover without full-file retries.

## Capabilities

### New Capabilities
- `token-efficient-code-io`: High-efficiency code read/write operations for agent workflows with strict token budgets and safe, minimal edits.

### Modified Capabilities
- None.

## Impact

- Affected areas: MCP server tools for code access, request/response contracts, and agent-facing documentation.
- Likely touched code: tool handlers for read/write operations, schema validation, and conflict/error reporting paths.
- External impact: improved agent throughput and lower token usage for codebase interaction.
