# flashgrep Development Guidelines

Auto-generated from all feature plans. Last updated: 2026-03-06

## Active Technologies
- Runtime session metadata payloads, file-based docs/skills artifacts, existing local metadata DB for operational checks (001-enforce-native-policy)
- Rust edition 2021 + existing `async-openai`, `serde_json`, `tokio`, `clap`, MCP stack (`mcp-server`, `mcp-protocol`) (005-expand-ai-capabilities)
- local metadata DB (`.flashgrep/metadata.db`), runtime session metadata payloads, docs/skills artifacts (005-expand-ai-capabilities)

- Rust edition 2021 + `serde_json`, `sha2`, `tokio`, `clap`, existing MCP stack (`mcp-server`, `mcp-protocol`) (002-improve-edit-precision)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test; cargo clippy

## Code Style

Rust edition 2021: Follow standard conventions

## Recent Changes
- 005-expand-ai-capabilities: Added Rust edition 2021 + existing `async-openai`, `serde_json`, `tokio`, `clap`, MCP stack (`mcp-server`, `mcp-protocol`)
- 001-enforce-native-policy: Added Rust edition 2021 + `serde_json`, `sha2`, `tokio`, `clap`, existing MCP stack (`mcp-server`, `mcp-protocol`)

- 002-improve-edit-precision: Added Rust edition 2021 + `serde_json`, `sha2`, `tokio`, `clap`, existing MCP stack (`mcp-server`, `mcp-protocol`)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
