## 1. CLI Command Wiring

- [x] 1.1 Add a `version` subcommand to CLI argument parsing and help metadata.
- [x] 1.2 Route `flashgrep version`, `--version`, and `-V` through one shared version output path.

## 2. Version Metadata Output

- [x] 2.1 Implement a version info formatter that includes Flashgrep version, OS, and CPU architecture.
- [x] 2.2 Source metadata from stable Rust/Cargo values and keep output labels deterministic.

## 3. Validation and Regression Coverage

- [x] 3.1 Add/update CLI tests for `flashgrep version` and parity with `--version` and `-V` output.
- [x] 3.2 Add/update help text assertions to ensure the `version` command is discoverable.
- [x] 3.3 Run the project test suite sections relevant to CLI behavior and fix any regressions.
