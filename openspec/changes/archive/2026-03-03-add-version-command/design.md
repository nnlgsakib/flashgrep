## Context

Flashgrep currently exposes version information through `--version` / `-V`, but output is limited to the application version string. This creates friction in debugging because issue reports often require platform details (OS and CPU architecture). The CLI already has established command parsing and help behavior, so this change should extend existing patterns without introducing new dependencies.

## Goals / Non-Goals

**Goals:**
- Add a first-class `flashgrep version` command for discoverability and consistency with other subcommands.
- Expand version output to include runtime environment details required for support and bug triage.
- Keep output deterministic and simple for users and automation.

**Non-Goals:**
- Adding telemetry, remote diagnostics, or machine fingerprinting.
- Changing search/index behavior or daemon lifecycle commands.
- Introducing configurable output formats in this change.

## Decisions

- Reuse one shared version-printer path for `version`, `--version`, and `-V` to guarantee consistent output.
- Include only stable, low-cost fields: Flashgrep version, target/host OS, CPU architecture, and build profile when available.
- Keep output plain text with fixed labels to preserve readability and copy/paste utility.
- Avoid additional crates; rely on existing compile-time/runtime metadata available in Rust std/Cargo env values.

## Risks / Trade-offs

- [Risk] Slightly longer output may break scripts expecting a single-line version string. -> Mitigation: document behavior in help/spec and keep labels predictable.
- [Risk] Runtime OS naming may vary by platform conventions. -> Mitigation: use canonical Rust-provided identifiers where possible.
- [Trade-off] Not adding JSON output keeps scope small but limits immediate machine parsing flexibility. -> Mitigation: fixed key-like labels make future structured output easier.
