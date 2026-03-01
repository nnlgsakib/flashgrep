# Grep/Glob Production Parity Matrix

This matrix tracks replacement-critical behavior for Flashgrep parity with common grep/glob workflows.

## Grep Compatibility

| Capability | Traditional expectation | Flashgrep command/option | Status |
|---|---|---|---|
| Fixed-string matching | `grep -F` matches literal tokens | `flashgrep query --fixed-string ...` | Implemented |
| Regex matching | `grep -E` style expressions | `flashgrep query --mode regex` | Implemented |
| Case-insensitive match | `grep -i` | `flashgrep query --ignore-case` | Implemented |
| Include/exclude path scope | scoped repo search | `--include`, `--exclude` | Implemented |
| Context output | `grep -C N` | `--context N` | Implemented |
| No-match exit code | exit 1 | `flashgrep query` returns exit 1 when no hits | Implemented |
| Failure exit code | exit 2+ for operational failure | `flashgrep` uses typed non-zero error codes | Implemented |

## Glob Compatibility

| Capability | Traditional expectation | Flashgrep command/option | Status |
|---|---|---|---|
| Recursive wildcard | `**` style recursion | `flashgrep files --pattern` | Implemented |
| Character classes | `[ab]` class patterns | `flashgrep files --pattern` | Implemented |
| Include hidden paths | optional dotfile matching | `--include-hidden` | Implemented |
| Exclude patterns | omit tree segments | `--exclude` | Implemented |
| Extension filters | suffix-based filtering | `--ext` | Implemented |
| Deterministic windows | stable pagination | `--sort-by`, `--sort-order`, `--offset`, `--limit` | Implemented |

## Filesystem Operations

| Capability | Requirement | Flashgrep command/option | Status |
|---|---|---|---|
| Create file/dir | deterministic creation | `flashgrep fs create` | Implemented |
| List directory | machine-readable listing | `flashgrep fs list` | Implemented |
| Stat metadata | stable fields for scripts | `flashgrep fs stat` | Implemented |
| Copy/move | conflict + overwrite control | `flashgrep fs copy/move --overwrite` | Implemented |
| Remove safety | dry-run + force controls | `flashgrep fs remove --dry-run --force` | Implemented |

## Exit Status Contract

- `0`: command successful with at least one match (for `query`) or successful operation.
- `1`: query completed with no matches.
- `2+`: operational failure (validation, IO, index, watcher, MCP, etc).
