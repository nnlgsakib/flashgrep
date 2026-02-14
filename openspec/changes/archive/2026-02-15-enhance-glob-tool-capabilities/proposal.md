## Why

Current glob behavior is too basic for advanced filtering and discovery workflows, forcing users and agents to chain extra steps and lose performance. We need a richer, native glob feature set so file discovery can be fast, expressive, and one-pass.

## What Changes

- Extend glob capabilities to support advanced include/exclude filters (extensions, directories, depth, hidden files, and path patterns).
- Add optional sorting and limiting controls tuned for fast, predictable file exploration.
- Add richer query options (for example case sensitivity and explicit recursion behavior) while preserving current defaults for backward compatibility.
- Ensure glob results remain efficient for large repositories and align with agent workflows that prioritize low overhead.
- Update docs and examples to cover the full glob feature set and recommended usage patterns.

## Capabilities

### New Capabilities
- `glob-query-enhancements`: Full-featured glob query and filtering behavior for efficient file discovery in large codebases.

### Modified Capabilities
- `mcp-server`: Expand MCP-level glob tool contract and response options to expose advanced filtering/sorting behavior.
- `ai-agent-documentation`: Document advanced glob options and guidance for efficient discovery workflows.

## Impact

- Affected code: MCP tool handlers and schemas, file discovery/search plumbing, and related tests.
- Affected docs: `skills/SKILL.md`, README MCP method docs, and examples.
- External impact: more powerful single-call glob discovery with improved speed and reduced follow-up calls.
