## Why

Flashgrep is optimized for very fast indexed search, but this speed is mostly exposed through MCP integrations and not fully available through direct CLI workflows. Users want grep/glob-like command-line operations that are faster and more index-aware than traditional filesystem scanning tools.

## What Changes

- Add CLI search commands that use Flashgrep's existing index and metadata engine for low-latency local queries.
- Add command-line equivalents for common grep/glob workflows (text query, file listing, symbol lookup, targeted code slices).
- Add filtering and output controls to support script-friendly usage and interactive terminal usage.
- Add clear command help and examples so users can replace repetitive grep/glob usage with indexed commands.

## Capabilities

### New Capabilities
- `cli-search-commands`: Provide grep/glob-like CLI commands powered by Flashgrep's indexed search core.

### Modified Capabilities
- `cli-interface`: Extend command surface and help text for fast indexed search operations.
- `search-engine`: Add CLI-facing result shaping requirements (limits, filtering, and consistent output formatting).

## Impact

- **Modified code areas**: `src/cli/`, `src/search/`, and argument/output formatting paths.
- **Docs**: CLI usage/help updates and command examples.
- **Performance/UX**: Faster repeated search workflows versus traditional grep/glob scans by using prebuilt indexes.
